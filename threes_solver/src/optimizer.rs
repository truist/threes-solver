use crate::algo::{Algos, WeightedAlgo};
use crate::solver;

use rng_util::RngType;
use threes_simulator::game_state::GameState;

use std::sync::Arc;
use std::thread;

use cmaes::{CMAESOptions, DVector};
use strum::{EnumCount, IntoEnumIterator};

#[cfg(debug_assertions)]
pub const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
pub const GAMES_PER_TEST: usize = 5_000;

pub fn find_optimal_weights(
    rng: &mut RngType,
    seed: u64,
    profiling: bool,
) -> cmaes::TerminationData {
    let threads = num_cpus::get_physical();

    let calc = |weights: &DVector<f64>| test_weighted_algo_set(weights, rng, threads);

    let max_generations = if profiling { 3 } else { 100 };
    let mut cmaes_options = CMAESOptions::new(vec![1.0; Algos::COUNT], 0.5)
        .mode(cmaes::Mode::Maximize)
        .seed(seed)
        .tol_x(1e-1)
        .tol_stagnation(50)
        .max_generations(max_generations)
        .enable_plot(cmaes::PlotOptions::new(0, false));

    // doing this annoying step to get a print for each generation
    let lambda = cmaes_options.population_size;
    cmaes_options = cmaes_options.enable_printing(lambda);

    println!(
        "Simulating {} games per test with {threads} threads",
        GAMES_PER_TEST
    );

    let mut cmaes_state = cmaes_options.build(calc).unwrap();

    let result = cmaes_state.run();

    cmaes_state
        .get_plot()
        .unwrap()
        .save_to_file("plot.png", true)
        .unwrap();

    result
}

pub fn test_weighted_algo_set(weights: &DVector<f64>, rng: &mut RngType, threads: usize) -> f64 {
    let algos = Arc::new(
        Algos::iter()
            .zip(weights.iter())
            .map(|(algo, &weight)| WeightedAlgo { algo, weight })
            .collect(),
    );

    let mut workers = vec![];
    for worker in 0..threads {
        let algos = Arc::clone(&algos);
        let mut worker_rng = rng_util::derive_worker_rng(rng, worker);

        let handle = thread::spawn(move || {
            let mut thread_moves = 0;

            // OK if this doesn't divide evenly; it will be close enough, and deterministic
            for _ in 0..GAMES_PER_TEST / threads {
                let (moves, _final_state) = solver::play(
                    GameState::initialize(&mut worker_rng),
                    &algos,
                    &mut worker_rng,
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
