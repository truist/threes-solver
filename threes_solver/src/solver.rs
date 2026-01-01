use crossterm::style::Stylize;
use strum::IntoEnumIterator;

use rng_util::RngType;
use threes_simulator::game_state::{Direction, GameState};

use crate::algo::WeightedAlgo;

pub fn play(
    mut game_state: GameState,
    algos: &Vec<WeightedAlgo>,
    lookahead_depth: usize,
    all_insertion_points: bool,
    rng: &mut RngType,
    verbose: bool,
) -> (usize, GameState) {
    let mut shifts = 0;

    if verbose {
        let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
        println!("Lookahead depth {lookahead_depth}; evaluating {insertion_point_desc} insertion point(s) per shift");
        println!("Initial state: {game_state}\n");
    }

    loop {
        let dir = choose_direction(
            &game_state,
            &algos,
            lookahead_depth,
            all_insertion_points,
            rng,
            verbose,
        );
        let new_state = game_state.shift(dir, true, rng);
        match new_state {
            Some(gs) => {
                game_state = gs;
                shifts += 1;

                if verbose {
                    print!("Shifted {dir}: ");
                    println!("\n{game_state}\n");
                }
            }
            None => {
                return (shifts, game_state);
            }
        }
    }
}

struct AlgoScore<'a> {
    weighted_algo: &'a WeightedAlgo,
    average_score: f64,
    all_scores: Vec<f64>,
}

struct Shift<'a> {
    direction: Direction,
    could_shift: bool,
    total_score: f64,
    algo_scores: Vec<AlgoScore<'a>>,
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
    all_insertion_points: bool,
    rng: &mut RngType,
    verbose: bool,
) -> Direction {
    let mut shifts: Vec<Shift> = Direction::iter()
        .map(|direction| {
            let mut could_shift = false;
            let mut shift_score = 0.0;
            let mut algo_scores: Vec<AlgoScore> = vec![];

            let shift_states = gen_shifted_states(game_state, direction, all_insertion_points, rng);
            if shift_states.len() > 0 {
                could_shift = true;
                shift_score = score_shift_states(
                    shift_states,
                    weighted_algos,
                    lookahead_depth,
                    rng,
                    &mut algo_scores,
                    verbose,
                );
            }

            Shift {
                direction,
                could_shift,
                total_score: shift_score,
                algo_scores,
            }
        })
        .collect();

    if verbose {
        print_verbose(&shifts);
    }

    choose_best_shift(&mut shifts)
}

fn gen_shifted_states(
    game_state: &GameState,
    direction: Direction,
    all_insertion_points: bool,
    rng: &mut RngType,
) -> Vec<GameState> {
    if all_insertion_points {
        game_state.shift_all(direction, rng)
    } else {
        let dir_state = game_state.shift(direction, true, rng);
        if let Some(actual_dir_state) = dir_state {
            vec![actual_dir_state]
        } else {
            vec![]
        }
    }
}

fn score_shift_states<'a>(
    shift_states: Vec<GameState>,
    weighted_algos: &'a [WeightedAlgo],
    lookahead_depth: usize,
    rng: &mut RngType,
    algo_scores: &mut Vec<AlgoScore<'a>>,
    verbose: bool,
) -> f64 {
    let mut all_algos_total = 0.0;

    let shift_states = extra_lookahead(shift_states, lookahead_depth, rng);

    for weighted_algo in weighted_algos.iter() {
        let weighted_scores: Vec<f64> = shift_states
            .iter()
            .map(|state| weighted_algo.score(&state))
            .collect();

        let average_score = if weighted_scores.len() > 0 {
            weighted_scores.iter().sum::<f64>() / weighted_scores.len() as f64
        } else {
            0.0
        };
        all_algos_total += average_score;

        if verbose {
            algo_scores.push(AlgoScore {
                weighted_algo,
                average_score,
                all_scores: weighted_scores,
            })
        }
    }

    all_algos_total
}

// This takes the original lookahead states and replaces them with
// the new states that come from trying each of the four possible
// shifts from the parent state. Then repeat, to the specified
// depth.
//
// It can all be wrapped up into a single group of resulting states
// because the caller is just trying to get a single total score
// for a single possible shift direction.
//
// However, this ends up averaging all the resulting viable moves,
// which might not be the best strategy. Maybe we should only use
// the "best possible" from each lookahead depth, and/or maybe
// ignoring the number of impossible shifts is a bad strategy.
//
// So more work is required here.
//
fn extra_lookahead(
    shift_states: Vec<GameState>,
    lookahead_depth: usize,
    rng: &mut RngType,
) -> Vec<GameState> {
    if lookahead_depth < 2 {
        return shift_states;
    }

    let mut deeper_states = shift_states;
    for _ in 2..=lookahead_depth {
        deeper_states = deeper_states
            .iter()
            .flat_map(|state| {
                Direction::iter()
                    .filter_map(|direction| state.shift(direction, false, rng))
                    .collect::<Vec<GameState>>()
            })
            .collect();
    }

    deeper_states
}

fn choose_best_shift(shifts: &mut Vec<Shift>) -> Direction {
    shifts.sort_by(|a, b| {
        a.could_shift
            .cmp(&b.could_shift)
            .then_with(|| a.total_score.total_cmp(&b.total_score)) // total_cmp to get totally-deterministic behavior
    });

    shifts.last().unwrap().direction
}

fn print_verbose(shifts: &Vec<Shift>) {
    println!("Considered these shifts:");

    for shift in shifts {
        if !shift.could_shift {
            println!("  {} (can't)", shift.direction);
        } else {
            println!("  {} ({}): ", shift.direction, fmt_f64(&shift.total_score));

            for algo_score in shift.algo_scores.iter() {
                let suffix =
                    render_score_list_if_unequal(&algo_score.all_scores, algo_score.average_score);

                println!(
                    "    {}: {}{}",
                    algo_score.weighted_algo,
                    fmt_f64(&algo_score.average_score),
                    suffix,
                );
            }
        }
    }
    println!("");
}

fn render_score_list_if_unequal(all_scores: &Vec<f64>, average_score: f64) -> String {
    let all_equal = all_scores
        .first()
        .map(|first| all_scores.iter().all(|score| score == first))
        .unwrap_or(true);

    if all_equal {
        String::from("")
    } else {
        let threshold = (average_score * 0.25).abs();
        let score_list = all_scores
            .iter()
            .map(|score| {
                let formatted = fmt_f64(score);
                if (score - average_score).abs() > threshold {
                    formatted.red().to_string()
                } else {
                    formatted.yellow().to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(" ({})", score_list)
    }
}

// strip trailing 0s and then a trailing . if that's all that's left
pub fn fmt_f64(val: &f64) -> String {
    format!("{:.3}", val)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
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
        let (shifts, final_state) = play(game_state, &algos, 1, false, &mut rng, false);

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

        let dir = choose_direction(&game_state, &weighted_algos, 1, false, &mut rng, false);
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
            let dir = choose_direction(&game_state, &weighted_algos, 1, true, &mut rng, true);
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
        let dir = choose_direction(&game_state, &algos, 1, false, &mut rng, false);
        assert_eq!(Direction::Right, dir, "With Empties strong, the best direction was right");

        // now swap the weights
        let algos: Vec<WeightedAlgo> = vec![
            WeightedAlgo { algo: Algos::Empties.to_algo(), weight: 1.0 },
            WeightedAlgo { algo: Algos::Merges.to_algo(), weight: 100.0 },
        ];
        let dir = choose_direction(&game_state, &algos, 1, false, &mut rng, false);
        assert_eq!(Direction::Left, dir, "With Merges strong, the best direction was left");
    }
}
