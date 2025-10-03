use rand::rngs::ThreadRng;
use rand::thread_rng;

use threes_simulator::game_state::{Direction, GameState};

pub fn solve() {
    let mut rng = thread_rng();
    let mut game_state = GameState::initialize(&mut rng);

    let original_score = score_state(&Some(game_state.clone()));
    println!("ORIGINAL, {}:\n{}", original_score, game_state);
    println!("");

    let mut moves = 0;
    loop {
        let (_score, new_state, _dir) = choose_move(&game_state, &mut rng);
        match new_state {
            Some(gs) => {
                // println!("CHOSEN ({score}): {dir}\n{gs}");
                game_state = gs;
                moves += 1;
            }
            None => {
                let sum = game_state
                    .get_grid()
                    .iter()
                    .map(|&card| card as u32)
                    .sum::<u32>();
                println!("FINAL (sum: {sum}, moves: {moves}):\n{game_state}");
                break;
            }
        }
    }
}

fn choose_move(game_state: &GameState, rng: &mut ThreadRng) -> (f64, Option<GameState>, Direction) {
    let mut moves: Vec<(f64, Option<GameState>, Direction)> = Direction::ALL
        .iter()
        .map(|&dir| {
            let state = game_state.shift(dir, rng);
            let score = score_state(&state);
            (score, state, dir)
        })
        .collect();
    moves.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    moves.pop().unwrap()
}

fn score_state(game_state: &Option<GameState>) -> f64 {
    if let Some(game_state) = game_state {
        let grid = game_state.get_grid();

        grid.iter()
            .map(|&card| if card > 0 { 0 } else { 1 })
            .sum::<u8>() as f64
    } else {
        0.0
    }
}
