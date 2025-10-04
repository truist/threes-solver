use rand::rngs::ThreadRng;

use crate::algo::Algos;
use threes_simulator::game_state::{Direction, GameState};

pub fn play(mut game_state: GameState, rng: &mut ThreadRng) -> (usize, GameState) {
    let algos = vec![Algos::Empties];

    let mut moves = 0;
    loop {
        let (_score, new_state, _dir) = choose_move(&game_state, &algos, rng);
        match new_state {
            Some(gs) => {
                // println!("CHOSEN ({_score}): {_dir}\n{gs}");
                game_state = gs;
                moves += 1;
            }
            None => {
                return (moves, game_state);
            }
        }
    }
}

fn choose_move(
    game_state: &GameState,
    algos: &Vec<Algos>,
    rng: &mut ThreadRng,
) -> (f64, Option<GameState>, Direction) {
    let mut moves: Vec<(f64, Option<GameState>, Direction)> = Direction::ALL
        .iter()
        .map(|&dir| {
            let state = game_state.shift(dir, rng);
            let score = algos.iter().map(|algo| algo.score(&state, &dir, &0)).sum();
            (score, state, dir)
        })
        .collect();

    moves.sort_by(|a, b| {
        a.1.is_some()
            .cmp(&b.1.is_some())
            .then_with(|| a.0.partial_cmp(&b.0).unwrap())
    });

    // println!("All moves: {:#?}", moves.clone());

    moves.pop().unwrap()
}

pub fn score_state(game_state: &Option<GameState>) -> f64 {
    if let Some(game_state) = game_state {
        let grid = game_state.get_grid();

        grid.iter()
            .map(|&card| if card > 0 { 0 } else { 1 })
            .sum::<u8>() as f64
    } else {
        0.0
    }
}

/************ tests *************/

/* test cases
 *  takes highest-value moves (for some known scoring algo)
 *  gets the right result (for some known scoring algo)
 *  specific scoring algos
 */

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_play() {
        let mut rng = thread_rng();
        let game_state = GameState::initialize(&mut rng);
        let (moves, final_state) = super::play(game_state, &mut rng);

        assert!(moves > 0, "it played at least one move");

        assert!(
            !final_state.get_grid().contains(&0),
            "all the board spaces are filled"
        );

        for each in Direction::ALL
            .iter()
            .map(|&dir| final_state.shift(dir, &mut rng))
        {
            if let None = each {
            } else {
                panic!("It was still possible to shift in some direction")
            }
        }
    }
}
