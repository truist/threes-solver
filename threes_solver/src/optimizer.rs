use crate::algo::{Algos, WeightedAlgo};
use crate::solver;

use threes_simulator::game_state::GameState;

use rand::rngs::ThreadRng;
use rand::thread_rng;
use strum::{EnumCount, IntoEnumIterator};

use cmaes::{CMAESOptions, DVector};

#[cfg(debug_assertions)]
const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
const GAMES_PER_TEST: usize = 5_000;

pub fn find_optimal_weights() -> cmaes::TerminationData {
    let mut rng = thread_rng();

    let calc = |weights: &DVector<f64>| test_weighted_algo_set(weights, &mut rng);

    let mut cmaes_config = CMAESOptions::new(vec![1.0; Algos::COUNT], 0.5)
        .mode(cmaes::Mode::Maximize)
        .tol_stagnation(50)
        .max_generations(50)
        .enable_printing(7)
        .enable_plot(cmaes::PlotOptions::new(0, false))
        .build(calc)
        .unwrap();

    let result = cmaes_config.run();

    cmaes_config
        .get_plot()
        .unwrap()
        .save_to_file("plot.png", true)
        .unwrap();

    result
}

pub fn test_weighted_algo_set(weights: &DVector<f64>, rng: &mut ThreadRng) -> f64 {
    let algos = Algos::iter()
        .zip(weights.iter())
        .map(|(algo, &weight)| WeightedAlgo { algo, weight })
        .collect();

    let mut total_moves = 0;

    for _ in 0..GAMES_PER_TEST {
        let (moves, _final_state) = solver::play(GameState::initialize(rng), &algos, rng);
        total_moves += moves;
    }

    total_moves as f64
}
