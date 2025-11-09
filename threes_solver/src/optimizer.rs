use crate::algo::{Algos, WeightedAlgo};
use crate::solver;

use rng_util::AnyRng;
use threes_simulator::game_state::GameState;

use strum::{EnumCount, IntoEnumIterator};

use cmaes::{CMAESOptions, DVector};

#[cfg(debug_assertions)]
pub const GAMES_PER_TEST: usize = 100;
#[cfg(not(debug_assertions))]
pub const GAMES_PER_TEST: usize = 5_000;

pub fn find_optimal_weights<R: AnyRng>(
    rng: &mut R,
    seed: u64,
    profiling: bool,
) -> cmaes::TerminationData {
    let calc = |weights: &DVector<f64>| test_weighted_algo_set(weights, rng);

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

    println!("Simulating {} games per test", GAMES_PER_TEST);

    let mut cmaes_state = cmaes_options.build(calc).unwrap();

    let result = cmaes_state.run();

    cmaes_state
        .get_plot()
        .unwrap()
        .save_to_file("plot.png", true)
        .unwrap();

    result
}

pub fn test_weighted_algo_set<R: AnyRng>(weights: &DVector<f64>, rng: &mut R) -> f64 {
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
