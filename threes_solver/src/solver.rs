use strum::IntoEnumIterator;

use crate::algo::WeightedAlgo;
use rng_util::RngType;
use threes_simulator::game_state::{Direction, GameState};

pub fn play(
    mut game_state: GameState,
    algos: &Vec<WeightedAlgo>,
    rng: &mut RngType,
    verbose: bool,
) -> (usize, GameState) {
    let mut moves = 0;

    if verbose {
        println!("Initial state: {game_state}\n");
    }

    loop {
        let dir = choose_move(&game_state, &algos, rng, verbose);
        let new_state = game_state.shift(dir, rng);
        match new_state {
            Some(gs) => {
                game_state = gs;
                moves += 1;

                if verbose {
                    print!("Move {moves} was {dir}: ");
                    println!("\n{game_state}\n");
                }
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
    rng: &mut RngType,
    verbose: bool,
) -> Direction {
    // perform all four moves
    // note that for bonus cards, this will pick one, but the "real" move might get a different one
    let mut moves: Vec<(f64, Option<GameState>, Direction, Vec<(&WeightedAlgo, f64)>)> = vec![];
    for dir in Direction::iter() {
        let dir_state = game_state.shift(dir, rng);

        let mut total_score = 0.0;
        let mut algo_scores: Vec<(&WeightedAlgo, f64)> = vec![];
        for algo in algos.iter() {
            let algo_score = algo.score(&dir_state, &dir);
            algo_scores.push((algo, algo_score));

            total_score += algo_score;
        }

        moves.push((total_score, dir_state, dir, algo_scores));
    }

    // sort by which moves succeeded, and then which of those has the best total_score
    moves.sort_by(|a, b| {
        a.1.is_some()
            .cmp(&b.1.is_some())
            .then_with(|| a.0.total_cmp(&b.0)) // total_cmp to get totally-deterministic behavior
    });

    // return the direction with the best total_score
    let answer = moves.last().unwrap().2;

    if verbose {
        println!("Considered these moves:");
        for mov in moves {
            if let Some(_) = mov.1 {
                print!("  {} ({}): ", mov.2, mov.0);
                for algo_score in mov.3 {
                    print!("{:?}: {}; ", algo_score.0.algo, algo_score.1);
                }
                println!("");
            } else {
                println!("  {} (can't)", mov.2);
            }
        }
        println!("");
    }

    answer
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo::Algos;

    use rng_util::test_rng;
    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;

    pub fn initialize_algos() -> Vec<WeightedAlgo> {
        Algos::iter()
            .map(|algo| WeightedAlgo { algo, weight: 1.0 })
            .collect()
    }

    #[test]
    fn test_play() {
        let mut rng = test_rng();
        let game_state = GameState::initialize(&mut rng);
        let algos = initialize_algos();
        let (moves, final_state) = super::play(game_state, &algos, &mut rng, false);

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
        let mut rng = test_rng();
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

        let dir = choose_move(&game_state, &algos, &mut rng, false);
        assert_eq!(Direction::Left, dir, "the best move was left");
    }

    // TODO: need tests that the weights are actually used in choosing moves
}
