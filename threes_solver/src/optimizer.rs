use crate::algo::WeightedAlgo;
use crate::solver;

use threes_simulator::game_state::GameState;

use rand::rngs::ThreadRng;
use rand::thread_rng;

#[cfg(debug_assertions)]
const GAMES: usize = 1_000;
#[cfg(not(debug_assertions))]
const GAMES: usize = 10_000;

pub fn find_optimal_weights() -> Vec<WeightedAlgo> {
    let mut rng = thread_rng();
    let algos = WeightedAlgo::initialize_all();
    let _score = test_weighted_algo_set(&algos, &mut rng);
    algos
}

pub fn test_weighted_algo_set(algos: &Vec<WeightedAlgo>, rng: &mut ThreadRng) -> usize {
    let mut total_moves = 0;

    for _ in 0..GAMES {
        let (moves, _final_state) = solver::play(GameState::initialize(rng), &algos, rng);
        total_moves += moves;
    }

    total_moves
}
