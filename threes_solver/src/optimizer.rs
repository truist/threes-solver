use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use cmaes::{CMAESOptions, DVector, TerminationData, CMAES};
use jiff::{Unit, Zoned};

use rng_util::RngType;
use threes_simulator::game_state::{Card, GameState};

use crate::algo::WeightedAlgo;
use crate::solver;

#[cfg(debug_assertions)]
pub const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
pub const GAMES_PER_TEST: usize = 5_000;

pub struct Optimizer<'a> {
    games_per_test: usize,
    threads: usize,

    rng: &'a RngType,
    seed: u64,
    lookahead_depth: usize,
    evaluate_all_insertion_points: bool,

    algos_count: usize,
    max_generations: usize,
    tol_x: f64,
    tol_stagnation: usize,
}

trait WeightTester: Fn(&DVector<f64>) -> f64 {}
impl<T> WeightTester for T where T: Fn(&DVector<f64>) -> f64 {}

impl<'a> Optimizer<'a> {
    pub fn new(
        rng: &'a RngType,
        seed: u64,
        max_threads: usize,
        lookahead_depth: usize,
        evaluate_all_insertion_points: bool,
        profiling: bool,
        rough: bool,
    ) -> Self {
        let mut threads = if profiling {
            1
        } else {
            num_cpus::get_physical()
        };
        if max_threads > 0 {
            threads = threads.min(max_threads);
        }

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

        Self {
            games_per_test: GAMES_PER_TEST,
            threads,

            rng,
            seed,
            lookahead_depth,
            evaluate_all_insertion_points,

            algos_count,
            max_generations,
            tol_x,
            tol_stagnation,
        }
    }

    pub fn find_optimal_weights(&self) -> cmaes::TerminationData {
        self.print_config();

        self.run_cmaes(self.configure_cmaes())
    }

    fn print_config(&self) {
        println!(
            "Starting run at {}",
            Zoned::now().round(Unit::Second).unwrap()
        );

        println!("Stop conditions:");
        println!("  max_generations: {}", self.max_generations);
        println!("  tol_x: {}", self.tol_x);
        println!("  tol_stagnation: {}", self.tol_stagnation);

        let insertion_point_desc = if self.evaluate_all_insertion_points {
            "all"
        } else {
            "1"
        };
        println!(
        "Simulating {} games per test with {} threads, with lookahead {}, evaluating {} insertion point(s) per shift",
        self.games_per_test, self.threads, self.lookahead_depth, insertion_point_desc
    );
    }

    fn configure_cmaes(&self) -> CMAES<impl WeightTester> {
        let mut cmaes_options = CMAESOptions::new(vec![1.0; self.algos_count], 0.5)
            .mode(cmaes::Mode::Maximize)
            .seed(self.seed)
            .tol_x(self.tol_x)
            .tol_stagnation(self.tol_stagnation)
            .max_generations(self.max_generations)
            .enable_plot(cmaes::PlotOptions::new(0, false));

        // get a print for each generation
        let lambda = cmaes_options.population_size;
        cmaes_options = cmaes_options.enable_printing(lambda);

        cmaes_options.build(self.interruptable_tester()).unwrap()
    }

    fn run_cmaes(&self, mut cmaes_state: CMAES<impl WeightTester>) -> TerminationData {
        let start = Instant::now();

        let result = cmaes_state.run();

        let elapsed = start.elapsed().as_secs();
        let games = cmaes_state.function_evals() * self.games_per_test;
        let game_rate = games as u64 / elapsed;

        println!("{games} games in {elapsed}s; {game_rate} games/s\n");

        cmaes_state
            .get_plot()
            .unwrap()
            .save_to_file("run_logs/plot.png", true)
            .unwrap();

        result
    }

    fn interruptable_tester(&self) -> impl WeightTester {
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
                self.test_weighted_algo_set(weights)
            }
        };

        calc
    }

    fn test_weighted_algo_set(&self, weights: &DVector<f64>) -> f64 {
        let weighted_algos = Arc::new(
            crate::algo::build_all_algos()
                .into_iter()
                .zip(weights.iter())
                .map(|(algo, &weight)| WeightedAlgo { algo, weight })
                .collect(),
        );

        let workers = self.make_worker_threads(weighted_algos, false);

        let mut total_moves = 0;
        for handle in workers {
            total_moves += handle.join().unwrap().0;
        }

        total_moves as f64
    }

    pub fn make_worker_threads(
        &self,
        weighted_algos: Arc<Vec<WeightedAlgo>>,
        collect_high_cards: bool,
    ) -> Vec<JoinHandle<(usize, Vec<Card>)>> {
        let mut workers = vec![];

        // make copies of these so they can be moved into the thread
        let lookahead_depth = self.lookahead_depth;
        let evaluate_all_insertion_points = self.evaluate_all_insertion_points;
        let games_per_test = self.games_per_test;
        let threads = self.threads;

        for worker in 0..self.threads {
            let weighted_algos = Arc::clone(&weighted_algos);
            let mut worker_rng = rng_util::derive_worker_rng(self.rng, worker);

            let handle = thread::spawn(move || {
                let mut thread_moves = 0;
                let mut high_cards = vec![];

                // It's OK if this doesn't divide evenly; it will be close enough, and deterministic
                for _ in 0..games_per_test / threads {
                    let (moves, final_state) = solver::play(
                        GameState::initialize(&mut worker_rng),
                        weighted_algos.as_ref(),
                        lookahead_depth,
                        evaluate_all_insertion_points,
                        &mut worker_rng,
                        false,
                    );
                    thread_moves += moves;
                    if collect_high_cards {
                        high_cards.push(*final_state.high_card());
                    }
                }

                (thread_moves, high_cards)
            });

            workers.push(handle);
        }

        workers
    }
}
