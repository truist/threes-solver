mod algo;
mod solver;

use rand::thread_rng;
use std::time::Instant;

use crate::solver::*;

use threes_simulator::game_state::GameState;

pub fn solve() {
    let mut rng = thread_rng();
    let game_state = GameState::initialize(&mut rng);

    let original_score = score_state(&Some(game_state.clone()));
    println!("ORIGINAL, {}:\n{}", original_score, game_state);
    println!("");

    let start = Instant::now();
    let (moves, final_state) = play(game_state, &mut rng);
    let duration = start.elapsed();
    println!(
        "FINAL (moves: {}, time: {:?}, time per move: {:?}):\n{}",
        moves,
        duration,
        duration.div_f64(moves as f64),
        final_state,
    );
}
