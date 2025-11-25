use strum::IntoEnumIterator;

use crate::algo::{Algo, WeightedAlgo};
use rng_util::RngType;
use threes_simulator::game_state::{Direction, GameState};

pub fn play(
    mut game_state: GameState,
    algos: &Vec<WeightedAlgo<dyn Algo>>,
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
                    print!("Moved {dir}: ");
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
    weighted_algos: &Vec<WeightedAlgo<dyn Algo>>,
    rng: &mut RngType,
    verbose: bool,
) -> Direction {
    // Perform all four moves.
    // Note that a shift might have multiple possible locations for the next card,
    // and multiple possible values for the next card when it is a bonus card.
    // (So a max of 12 possible distinct outcomes, in each direction.)
    // This function (currently) will just test one random case for each direction,
    // and use that to decide which direction is best.
    // The actual shift performed by the caller might get different results.
    let mut moves: Vec<(
        f64,
        Option<GameState>,
        Direction,
        Vec<(&WeightedAlgo<dyn Algo>, f64)>,
    )> = vec![];

    for dir in Direction::iter() {
        let dir_state = game_state.shift(dir, rng);

        let mut total_score = 0.0;
        let mut algo_scores: Vec<(&WeightedAlgo<dyn Algo>, f64)> = vec![];
        for weighted_algo in weighted_algos.iter() {
            let algo_score = weighted_algo.score(&dir_state);
            algo_scores.push((weighted_algo, algo_score));

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
    use crate::algo::{Algo, Algos};

    use rng_util::test_rng;
    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;

    pub fn initialize_algos() -> Vec<WeightedAlgo<dyn Algo>> {
        crate::algo::build_all_algos()
            .into_iter()
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
        let weighted_algos: Vec<WeightedAlgo<dyn Algo>> = vec![WeightedAlgo {
            algo: Box::new(Algos::Empties),
            weight: 1.0,
        }];

        let board_state = BoardState::initialize_test_state([
            0, 0, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 3);

        let game_state = GameState::initialize_test_state(board_state, draw_pile, next);

        let dir = choose_move(&game_state, &weighted_algos, &mut rng, false);
        assert_eq!(Direction::Left, dir, "the best move was left");
    }

    #[test]
    #[rustfmt::skip]
    fn test_weights() {
        let mut rng = test_rng();
        let mut draw_pile = DrawPile::initialize_test_pile(vec![1, 1]);
        let next = draw_pile.draw(&mut rng);
        let board_state = BoardState::initialize_test_state([
            6, 0, 0, 0,
            0, 6, 3, 3,
            6, 0, 0, 0,
            0, 6, 0, 0,
        ], 3);
        let game_state = GameState::initialize_test_state(board_state, draw_pile, next);

        let algos: Vec<WeightedAlgo<dyn Algo>> = vec![
            WeightedAlgo { algo: Box::new(Algos::Empties), weight: 100.0 },
            WeightedAlgo { algo: Box::new(Algos::Merges), weight: 1.0 },
        ];
        let dir = choose_move(&game_state, &algos, &mut rng, false);
        assert_eq!(Direction::Right, dir, "With Empties strong, the best move was right");

        // now swap the weights
        let algos: Vec<WeightedAlgo<dyn Algo>> = vec![
            WeightedAlgo { algo: Box::new(Algos::Empties), weight: 1.0 },
            WeightedAlgo { algo: Box::new(Algos::Merges), weight: 100.0 },
        ];
        let dir = choose_move(&game_state, &algos, &mut rng, false);
        assert_eq!(Direction::Left, dir, "With Merges strong, the best move was left");

    }
}
