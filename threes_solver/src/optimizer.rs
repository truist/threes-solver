use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use cmaes::{CMAESOptions, DVector, TerminationData, CMAES};
use jiff::{Unit, Zoned};

use rng_util::RngType;
use threes_simulator::game_state::GameState;

use crate::algo::WeightedAlgo;
use crate::solver;

#[cfg(debug_assertions)]
pub const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
pub const GAMES_PER_TEST: usize = 5_000;

struct RunConfig<'a> {
    games_per_test: usize,
    threads: usize,

    rng: &'a RngType,
    seed: u64,
    evaluate_all_insertion_points: bool,

    algos_count: usize,
    max_generations: usize,
    tol_x: f64,
    tol_stagnation: usize,
}

fn init_config<'a>(
    rng: &'a RngType,
    seed: u64,
    evaluate_all_insertion_points: bool,
    profiling: bool,
    rough: bool,
) -> RunConfig<'a> {
    let threads = if profiling {
        1
    } else {
        num_cpus::get_physical()
    };

    let algos_count = crate::algo::build_all_algos().len();

    let max_generations = if profiling {
        3
    } else if rough {
        algos_count * 10
    } else {
        algos_count * 25
    };

    let tol_x = if rough { 0.2 } else { 0.1 };

    let tol_stagnation = if rough {
        (algos_count / 2).max(15)
    } else {
        (algos_count).max(30)
    };

    RunConfig {
        games_per_test: GAMES_PER_TEST,
        threads,

        rng,
        seed,
        evaluate_all_insertion_points,

        algos_count,
        max_generations,
        tol_x,
        tol_stagnation,
    }
}

fn print_config(run_config: &RunConfig) {
    println!(
        "Starting run at {}",
        Zoned::now().round(Unit::Second).unwrap()
    );

    println!("Stop conditions:");
    println!("  max_generations: {}", run_config.max_generations);
    println!("  tol_x: {}", run_config.tol_x);
    println!("  tol_stagnation: {}", run_config.tol_stagnation);

    let insertion_point_desc = if run_config.evaluate_all_insertion_points {
        "all"
    } else {
        "1"
    };
    println!(
        "Simulating {} games per test with {} threads, evaluating {} insertion point(s) per shift",
        run_config.games_per_test, run_config.threads, insertion_point_desc
    );
}

pub fn find_optimal_weights(
    rng: &mut RngType,
    seed: u64,
    evaluate_all_insertion_points: bool,
    profiling: bool,
    rough: bool,
) -> cmaes::TerminationData {
    let run_config = init_config(rng, seed, evaluate_all_insertion_points, profiling, rough);
    print_config(&run_config);

    run_cmaes(configure_cmaes(&run_config), &run_config)
}

trait WeightTester: Fn(&DVector<f64>) -> f64 {}
impl<T> WeightTester for T where T: Fn(&DVector<f64>) -> f64 {}

fn configure_cmaes(run_config: &RunConfig) -> CMAES<impl WeightTester> {
    let mut cmaes_options = CMAESOptions::new(vec![1.0; run_config.algos_count], 0.5)
        .mode(cmaes::Mode::Maximize)
        .seed(run_config.seed)
        .tol_x(run_config.tol_x)
        .tol_stagnation(run_config.tol_stagnation)
        .max_generations(run_config.max_generations)
        .enable_plot(cmaes::PlotOptions::new(0, false));

    // get a print for each generation
    let lambda = cmaes_options.population_size;
    cmaes_options = cmaes_options.enable_printing(lambda);

    cmaes_options
        .build(interruptable_tester(&run_config))
        .unwrap()
}

fn run_cmaes(mut cmaes_state: CMAES<impl WeightTester>, run_config: &RunConfig) -> TerminationData {
    let start = Instant::now();

    let result = cmaes_state.run();

    let elapsed = start.elapsed().as_secs();
    let games = cmaes_state.function_evals() * run_config.games_per_test;
    let game_rate = games as u64 / elapsed;

    println!("{games} games in {elapsed}s; {game_rate} games/s\n");

    cmaes_state
        .get_plot()
        .unwrap()
        .save_to_file("plot.png", true)
        .unwrap();

    result
}

fn interruptable_tester<'a>(run_config: &'a RunConfig) -> impl WeightTester + 'a {
    let stop = Arc::new(AtomicBool::new(false));

    {
        let handler_stop = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            if !handler_stop.load(Ordering::SeqCst) {
                eprintln!("Stopping gracefully; press Ctrl-C again to stop immediately...");
                handler_stop.store(true, Ordering::SeqCst);
            } else {
                std::process::exit(130); // standard SIGINT exit code
            }
        })
        .unwrap();
    }

    let calc = move |weights: &DVector<f64>| {
        if stop.load(Ordering::SeqCst) {
            f64::NAN
        } else {
            test_weighted_algo_set(weights, run_config)
        }
    };

    calc
}

fn test_weighted_algo_set(weights: &DVector<f64>, run_config: &RunConfig) -> f64 {
    let weighted_algos = Arc::new(
        crate::algo::build_all_algos()
            .into_iter()
            .zip(weights.iter())
            .map(|(algo, &weight)| WeightedAlgo { algo, weight })
            .collect(),
    );

    let workers = make_worker_threads(weighted_algos, run_config);

    let mut total_moves = 0;
    for handle in workers {
        total_moves += handle.join().unwrap();
    }

    total_moves as f64
}

fn make_worker_threads(
    weighted_algos: Arc<Vec<WeightedAlgo>>,
    run_config: &RunConfig,
) -> Vec<JoinHandle<usize>> {
    let mut workers = vec![];

    let threads = run_config.threads;
    let evaluate_all_insertion_points = run_config.evaluate_all_insertion_points;
    let games_per_test = run_config.games_per_test;

    for worker in 0..run_config.threads {
        let weighted_algos = Arc::clone(&weighted_algos);
        let mut worker_rng = rng_util::derive_worker_rng(run_config.rng, worker);

        let handle = thread::spawn(move || {
            let mut thread_moves = 0;

            // It's OK if this doesn't divide evenly; it will be close enough, and deterministic
            for _ in 0..games_per_test / threads {
                let (moves, _final_state) = solver::play(
                    GameState::initialize(&mut worker_rng),
                    weighted_algos.as_ref(),
                    evaluate_all_insertion_points,
                    &mut worker_rng,
                    false,
                );
                thread_moves += moves;
            }

            thread_moves
        });

        workers.push(handle);
    }

    workers
}
