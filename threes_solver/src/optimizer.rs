use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use cmaes::{CMAESOptions, DVector};
use jiff::{Unit, Zoned};

use rng_util::RngType;
use threes_simulator::game_state::GameState;

use crate::algo::WeightedAlgo;
use crate::solver;

#[cfg(debug_assertions)]
pub const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
pub const GAMES_PER_TEST: usize = 5_000;

pub fn find_optimal_weights(
    rng: &mut RngType,
    seed: u64,
    all_insertion_points: bool,
    profiling: bool,
    rough: bool,
) -> cmaes::TerminationData {
    let threads = if profiling {
        1
    } else {
        num_cpus::get_physical()
    };

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || {
            if !stop.load(Ordering::SeqCst) {
                eprintln!("Stopping gracefully; press Ctrl-C again to stop immediately...");
                stop.store(true, Ordering::SeqCst);
            } else {
                std::process::exit(130); // standard SIGINT exit code
            }
        })
        .unwrap();
    }
    let calc = |weights: &DVector<f64>| {
        if stop.load(Ordering::SeqCst) {
            f64::NAN
        } else {
            test_weighted_algo_set(weights, all_insertion_points, rng, threads)
        }
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

    let now = Zoned::now().round(Unit::Second).unwrap();
    println!("Starting run at {now}");

    println!("Stop conditions:");
    println!("  max_generations: {max_generations}");
    println!("  tol_x: {tol_x}");
    println!("  tol_stagnation: {tol_stagnation}");

    let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
    println!("Simulating {GAMES_PER_TEST} games per test with {threads} threads, evaluating {insertion_point_desc} insertion point(s) per shift");

    let mut cmaes_options = CMAESOptions::new(vec![1.0; algos_count], 0.5)
        .mode(cmaes::Mode::Maximize)
        .seed(seed)
        .tol_x(tol_x)
        .tol_stagnation(tol_stagnation)
        .max_generations(max_generations)
        .enable_plot(cmaes::PlotOptions::new(0, false));

    // doing this annoying step to get a print for each generation
    let lambda = cmaes_options.population_size;
    cmaes_options = cmaes_options.enable_printing(lambda);

    let mut cmaes_state = cmaes_options.build(calc).unwrap();

    let result = cmaes_state.run();

    cmaes_state
        .get_plot()
        .unwrap()
        .save_to_file("plot.png", true)
        .unwrap();

    result
}

pub fn test_weighted_algo_set(
    weights: &DVector<f64>,
    all_insertion_points: bool,
    rng: &mut RngType,
    threads: usize,
) -> f64 {
    let algos = crate::algo::build_all_algos();

    let weighted_algos = Arc::new(
        algos
            .into_iter()
            .zip(weights.iter())
            .map(|(algo, &weight)| WeightedAlgo { algo, weight })
            .collect(),
    );

    let mut workers = vec![];
    for worker in 0..threads {
        let weighted_algos = Arc::clone(&weighted_algos);
        let mut worker_rng = rng_util::derive_worker_rng(rng, worker);

        let handle = thread::spawn(move || {
            let mut thread_moves = 0;

            // OK if this doesn't divide evenly; it will be close enough, and deterministic
            for _ in 0..GAMES_PER_TEST / threads {
                let (moves, _final_state) = solver::play(
                    GameState::initialize(&mut worker_rng),
                    &weighted_algos,
                    all_insertion_points,
                    &mut worker_rng,
                    false,
                );
                thread_moves += moves;
            }

            thread_moves
        });
        workers.push(handle);
    }

    let mut total_moves = 0;
    for handle in workers {
        total_moves += handle.join().unwrap();
    }

    total_moves as f64
}
