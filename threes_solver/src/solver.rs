use rand::rngs::ThreadRng;
use strum::IntoEnumIterator;

use crate::algo::WeightedAlgo;
use threes_simulator::game_state::{Direction, GameState};

pub fn play(
    mut game_state: GameState,
    algos: &Vec<WeightedAlgo>,
    rng: &mut ThreadRng,
) -> (usize, GameState) {
    let mut moves = 0;
    loop {
        let dir = choose_move(&game_state, &algos, rng);
        let new_state = game_state.shift(dir, rng);
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
    algos: &Vec<WeightedAlgo>,
    rng: &mut ThreadRng,
) -> Direction {
    // perform all four moves
    // note that for bonus cards, this will pick one, but the "real" move might get a different one
    let mut moves: Vec<(f64, Option<GameState>, Direction)> = Direction::iter()
        .map(|dir| {
            let state = game_state.shift(dir, rng);
            let score = algos
                .iter()
                .map(|weighted_algo| weighted_algo.score(&state, &dir))
                .sum();
            (score, state, dir)
        })
        .collect();

    // sort by which moves succeeded, and then which of those has the best score
    moves.sort_by(|a, b| {
        a.1.is_some()
            .cmp(&b.1.is_some())
            .then_with(|| a.0.partial_cmp(&b.0).unwrap())
    });

    // println!("All moves: {:#?}", moves.clone());

    // return the direction with the best score
    moves.pop().unwrap().2
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo::Algos;

    use rand::thread_rng;

    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;

    #[test]
    fn test_play() {
        let mut rng = thread_rng();
        let game_state = GameState::initialize(&mut rng);
        let algos = WeightedAlgo::initialize_all();
        let (moves, final_state) = super::play(game_state, &algos, &mut rng);

        assert!(moves > 0, "it played at least one move");

        assert!(
            !final_state.get_grid().contains(&0),
            "all the board spaces are filled"
        );

        for each in Direction::iter().map(|dir| final_state.shift(dir, &mut rng)) {
            if let None = each {
            } else {
                panic!("It was still possible to shift in some direction")
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_choose_move() {
        let mut rng = thread_rng();
        let mut draw_pile = DrawPile::initialize_test_pile(vec![1]);
        let next = draw_pile.draw(&mut rng);
        let algos = vec![WeightedAlgo { algo: Algos::Empties, weight: 1.0 }];
        //TODO need a test for multiple algos

        let board_state = BoardState::initialize_test_state([
            0, 0, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 3);

        let game_state = GameState::initialize_test_state(board_state, draw_pile, next);

        let dir = choose_move(&game_state, &algos, &mut rng);
        assert_eq!(Direction::Left, dir, "the best move was left");
    }

    // TODO: need tests that the weights are actually used in choosing moves
}
