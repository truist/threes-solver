mod algo;
mod solver;

use std::time::Instant;

use rand::thread_rng;
use strum::IntoEnumIterator;

use threes_simulator::game_state::GameState;

use crate::algo::Algos;
use crate::solver::*;

pub fn solve() {
    let algos: Vec<Algos> = Algos::iter().collect();
    println!("Running with {} algos", algos.len());

    let mut rng = thread_rng();
    let game_state = GameState::initialize(&mut rng);

    let original_score = score_state(&Some(game_state.clone()));
    println!("ORIGINAL, {}:\n{}", original_score, game_state);
    println!("");

    let start = Instant::now();
    let (moves, final_state) = play(game_state, algos, &mut rng);
    let duration = start.elapsed();
    println!(
        "FINAL (moves: {}, time: {:?}, time per move: {:?}):\n{}",
        moves,
        duration,
        duration.div_f64(moves as f64),
        final_state,
    );
}
