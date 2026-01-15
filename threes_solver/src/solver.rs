use strum::IntoEnumIterator;

use rng_util::RngType;
use threes_simulator::game_state::{Direction, GameState};

use crate::algo::WeightedAlgo;

use crate::verbose_util::{self, Verbose};

pub fn play(
    mut game_state: GameState,
    algos: &Vec<WeightedAlgo>,
    lookahead_depth: usize,
    all_insertion_points: bool,
    rng: &mut RngType,
    verbose: Verbose,
) -> (usize, GameState) {
    if !matches!(verbose, Verbose::Off) {
        let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
        println!("Lookahead depth {lookahead_depth}; evaluating {insertion_point_desc} insertion point(s) per shift");
        println!("Initial state: {game_state}\n");
    }
    let insertion_points = if all_insertion_points {
        InsertionPoints::All
    } else {
        InsertionPoints::One
    };

    let mut algo_summary_data = if !matches!(verbose, Verbose::Off) {
        vec![Vec::new(); algos.len()]
    } else {
        vec![]
    };
    let mut shifts = 0;

    loop {
        let dir = choose_direction(
            &game_state,
            &algos,
            lookahead_depth,
            &insertion_points,
            rng,
            &verbose,
            &mut algo_summary_data,
        );
        let new_state = game_state.shift(dir, true, rng);
        match new_state {
            Some(gs) => {
                game_state = gs;
                shifts += 1;

                if !matches!(verbose, Verbose::Off) {
                    println!("Shifted {dir}");
                    println!("\n{game_state}\n");
                }
            }
            None => {
                if !matches!(verbose, Verbose::Off) {
                    verbose_util::print_algo_summary(&algo_summary_data, algos);
                }
                return (shifts, game_state);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct AlgoScore<'a> {
    pub(crate) weighted_algo: &'a WeightedAlgo,
    pub(crate) score: f64,
}

#[derive(Debug)]
pub(crate) struct StateScores<'a> {
    pub(crate) algo_scores: Vec<AlgoScore<'a>>,
}

impl<'a> StateScores<'a> {
    fn sum(&self) -> f64 {
        self.algo_scores
            .iter()
            .map(|algo_score| algo_score.score)
            .sum()
    }
}

#[derive(Debug)]
pub(crate) struct Shift<'a> {
    pub(crate) direction: Direction,
    pub(crate) could_shift: bool,
    pub(crate) total_score: f64,
    pub(crate) state_scores: Vec<StateScores<'a>>,
}

enum InsertionPoints {
    All,
    One,
    // Zero,
}

// Perform all possible shifts and choose the best one.
// Note that a shift might have 4 possible insertion locations for the next card,
// and 3 possible values for the next card if it is a bonus card.
// So a max of 12 possible distinct outcomes, in each of 4 directions,
// for a total of 48 cases to evaluate.
// The actual shift performed by the caller will get just one of these cases.
fn choose_direction<'a>(
    game_state: &GameState,
    weighted_algos: &'a [WeightedAlgo],
    lookahead_depth: usize,
    insertion_points: &InsertionPoints,
    rng: &mut RngType,
    verbose: &Verbose,
    algo_summary_data: &mut Vec<Vec<f64>>,
) -> Direction {
    let mut shifts = score_directions(
        game_state,
        weighted_algos,
        lookahead_depth,
        insertion_points,
        rng,
        algo_summary_data,
    );
    let best_direction = shifts.last().unwrap().direction;

    verbose_util::print_verbose(verbose, &mut shifts);

    best_direction
}

// New plan, if lookahead_depth >= 2:
//   For each insertion_point_state:
//     Calculate all 4 directions, ignoring new insertion points (for now)
//       So each direction will have 0 or 1 states; no more than that
//       Calculate the score for each direction:
//         If we haven't hit max lookahead_depth yet, recurse
//           And use the result from our child, for this direction
//         If we have hit max lookahead_depth just calculate the score
//       Choose the best direction and return its score
//         Because the user would never choose the other directions, when they got to this point
fn score_directions<'a>(
    game_state: &GameState,
    weighted_algos: &'a [WeightedAlgo],
    lookahead_depth: usize,
    insertion_points: &InsertionPoints,
    rng: &mut RngType,
    algo_summary_data: &mut Vec<Vec<f64>>,
) -> Vec<Shift<'a>> {
    let mut shifts: Vec<Shift> = Direction::iter()
        .map(|direction| {
            let mut could_shift = false;
            let mut shift_score = 0.0;
            let mut insertion_point_scores: Vec<StateScores> = vec![];

            let insertion_point_states =
                gen_states_for_insertion_points(game_state, direction, insertion_points, rng);

            if insertion_point_states.len() > 0 {
                could_shift = true;

                insertion_point_scores = insertion_point_states
                    .iter()
                    .map(|state| {
                        score_state_with_lookahead(
                            &state,
                            weighted_algos,
                            lookahead_depth - 1,
                            rng,
                            algo_summary_data,
                        )
                    })
                    .collect();

                shift_score = insertion_point_scores
                    .iter()
                    .map(|state_scores| state_scores.sum())
                    .sum::<f64>()
                    / insertion_point_scores.len().max(1) as f64;
            }

            Shift {
                direction,
                could_shift,
                total_score: shift_score,
                state_scores: insertion_point_scores,
            }
        })
        .collect();

    shifts.sort_by(|a, b| {
        a.could_shift
            .cmp(&b.could_shift)
            .then_with(|| a.total_score.total_cmp(&b.total_score)) // total_cmp to get totally-deterministic behavior
    });

    shifts
}

fn gen_states_for_insertion_points(
    game_state: &GameState,
    direction: Direction,
    insertion_points: &InsertionPoints,
    rng: &mut RngType,
) -> Vec<GameState> {
    if let InsertionPoints::All = insertion_points {
        game_state.shift_all(direction, rng)
    } else {
        let dir_state = game_state.shift(
            direction,
            matches!(insertion_points, InsertionPoints::One),
            rng,
        );
        if let Some(actual_dir_state) = dir_state {
            vec![actual_dir_state]
        } else {
            vec![]
        }
    }
}

fn score_state_with_lookahead<'a>(
    game_state: &GameState,
    weighted_algos: &'a [WeightedAlgo],
    remaining_depth: usize,
    rng: &mut RngType,
    algo_summary_data: &mut Vec<Vec<f64>>,
) -> StateScores<'a> {
    if remaining_depth == 0 {
        score_state(game_state, weighted_algos, algo_summary_data)
    } else {
        let shifts = score_directions(
            game_state,
            weighted_algos,
            remaining_depth,
            &InsertionPoints::One, // good enough, in testing
            rng,
            algo_summary_data,
        );

        let best_direction = shifts.into_iter().last().unwrap();
        if best_direction.could_shift {
            best_direction.state_scores.into_iter().next().unwrap()
        } else {
            // no shifts downstream from us, so just score our current state
            score_state(game_state, weighted_algos, algo_summary_data)
        }
    }
}

fn score_state<'a>(
    game_state: &GameState,
    weighted_algos: &'a [WeightedAlgo],
    algo_summary_data: &mut Vec<Vec<f64>>,
) -> StateScores<'a> {
    if !algo_summary_data.is_empty() {
        for (algo_index, weighted_algo) in weighted_algos.iter().enumerate() {
            algo_summary_data[algo_index].push(weighted_algo.algo.score(game_state, None));
        }
    }

    StateScores {
        algo_scores: weighted_algos
            .iter()
            .map(|weighted_algo| AlgoScore {
                weighted_algo,
                score: weighted_algo.score(game_state),
            })
            .collect(),
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use rng_util::test_rng;
    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;

    use crate::algo::Algos;

    use super::*;

    pub fn initialize_algos() -> Vec<WeightedAlgo> {
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
        let (shifts, final_state) = play(game_state, &algos, 1, false, &mut rng, Verbose::Off);

        assert!(shifts > 0, "it played at least one shift");

        assert!(
            !final_state.get_grid().contains(&0),
            "all the board spaces are filled"
        );

        for each in Direction::iter().map(|dir| final_state.shift(dir, true, &mut rng)) {
            if let None = each {
            } else {
                panic!("It was still possible to shift in some direction")
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_choose_direction() {
        let mut rng = test_rng();
        let mut draw_pile = DrawPile::initialize_test_pile(vec![1]);
        let next = draw_pile.draw(&mut rng);

        let weighted_algos: Vec<WeightedAlgo> = vec![WeightedAlgo {
            algo: Algos::Empties.to_algo(),
            weight: 1.0,
        }];

        let board_state = BoardState::initialize_test_state([
            0, 0, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 3);
        let game_state = GameState::initialize_test_state(board_state, draw_pile, next);

        let dir = choose_direction(
            &game_state,
            &weighted_algos,
            1,
            &InsertionPoints::One,
            &mut rng,
            &Verbose::Off,
            &mut vec![],
        );
        assert_eq!(Direction::Left, dir, "the best direction was left");


        let mut draw_pile = DrawPile::initialize_test_pile(vec![3]);
        let next = draw_pile.draw(&mut rng);
        let weighted_algos: Vec<WeightedAlgo> = vec![WeightedAlgo {
            algo: Algos::Merges.to_algo(),
            weight: 1.0,
        }];
        // With this state, using only Merges, with a 3 next:
        //   - shifting left will create a new merge 2/4 times
        //   - shifting up will create a new merge 1/3 times
        //   - shifting right will create a new merge 0/2 times
        //   - shifting down will create a new merge 1/3 times
        // So we can use many iterations to test whether the algo always says to shift left;
        // if not, it didn't use the real average
        let board_state = BoardState::initialize_test_state([
            0, 3, 0, 0,
            0, 0, 6, 3,
            0, 0, 0, 3,
            0, 3, 0, 0,
        ], 3);
        let game_state = GameState::initialize_test_state(board_state, draw_pile, next);
        for i in 0..100 {
            let dir = choose_direction(
                &game_state,
                &weighted_algos,
                1,
                &InsertionPoints::All,
                &mut rng,
                &Verbose::Off,
                &mut vec![],
            );
            assert_eq!(Direction::Left, dir, "the best direction is always left ({i})");
        }
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

        let algos: Vec<WeightedAlgo> = vec![
            WeightedAlgo { algo: Algos::Empties.to_algo(), weight: 100.0 },
            WeightedAlgo { algo: Algos::Merges.to_algo(), weight: 1.0 },
        ];
        let dir = choose_direction(&game_state, &algos, 1, &InsertionPoints::One, &mut rng, &Verbose::Off, &mut vec![]);
        assert_eq!(Direction::Right, dir, "With Empties strong, the best direction was right");

        // now swap the weights
        let algos: Vec<WeightedAlgo> = vec![
            WeightedAlgo { algo: Algos::Empties.to_algo(), weight: 1.0 },
            WeightedAlgo { algo: Algos::Merges.to_algo(), weight: 100.0 },
        ];
        let dir = choose_direction(&game_state, &algos, 1, &InsertionPoints::One, &mut rng, &Verbose::Off, &mut vec![]);
        assert_eq!(Direction::Left, dir, "With Merges strong, the best direction was left");
    }
}
