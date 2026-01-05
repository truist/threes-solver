use crossterm::style::Stylize;
use strum::IntoEnumIterator;
use unicode_width::UnicodeWidthStr;

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
    if verbose {
        let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
        println!("Lookahead depth {lookahead_depth}; evaluating {insertion_point_desc} insertion point(s) per shift");
        println!("Initial state: {game_state}\n");
    }
    let insertion_points = if all_insertion_points {
        InsertionPoints::All
    } else {
        InsertionPoints::One
    };

    let mut algo_summary_data = if verbose {
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
            verbose,
            &mut algo_summary_data,
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
                if verbose {
                    print_algo_summary(&algo_summary_data, algos);
                }
                return (shifts, game_state);
            }
        }
    }
}

#[derive(Debug)]
struct AlgoScore<'a> {
    weighted_algo: &'a WeightedAlgo,
    score: f64,
}

#[derive(Debug)]
struct StateScores<'a> {
    algo_scores: Vec<AlgoScore<'a>>,
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
struct Shift<'a> {
    direction: Direction,
    could_shift: bool,
    total_score: f64,
    state_scores: Vec<StateScores<'a>>,
}

impl<'a> Shift<'a> {
    fn merge_algo_scores(&self) -> Vec<MergedAlgoScores<'a>> {
        let mut merged = vec![];
        if self.state_scores.len() == 0 {
            return merged;
        }

        let first_algo_scores = &self.state_scores.first().unwrap().algo_scores;
        for wai in 0..first_algo_scores.len() {
            let all_scores: Vec<f64> = self
                .state_scores
                .iter()
                .map(|state_score| state_score.algo_scores[wai].score)
                .collect();
            let average_score = all_scores.iter().sum::<f64>() / all_scores.len().max(1) as f64;

            let weighted_algo = first_algo_scores[wai].weighted_algo;

            merged.push(MergedAlgoScores {
                weighted_algo,
                average_score,
                all_scores,
            });
        }

        merged
    }
}

struct MergedAlgoScores<'a> {
    weighted_algo: &'a WeightedAlgo,
    average_score: f64,
    all_scores: Vec<f64>,
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
    verbose: bool,
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

    if verbose {
        print_verbose(&mut shifts);
    }

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

fn print_verbose(shifts: &mut Vec<Shift>) {
    const ALGO_COL_WIDTH: usize = 27;
    const NORM_COL_WIDTH: usize = 5;
    const WEIGHT_COL_WIDTH: usize = 5;
    const AVERAGE_COL_WIDTH: usize = 7;

    println!("Considered these shifts:");
    let algo_header = pad_to_width("Algo", ALGO_COL_WIDTH).blue();
    let norm_header = pad_to_width("Norm", NORM_COL_WIDTH).blue();
    let weight_header = pad_to_width("Weight", WEIGHT_COL_WIDTH).blue();
    let average_header = pad_to_width("Average", AVERAGE_COL_WIDTH).blue();
    let insertion_header = "Insertion variations".blue();
    println!(
        "    {}  {}  {}  {}  {}",
        algo_header, norm_header, weight_header, average_header, insertion_header,
    );

    shifts.reverse();
    let mut algo_order: Vec<*const WeightedAlgo> = Vec::new();
    for (shift_index, shift) in shifts.iter_mut().enumerate() {
        if !shift.could_shift {
            println!("  {} (can't)", shift.direction);
        } else {
            println!("  {} ({}): ", shift.direction, fmt_f64(&shift.total_score));

            let mut merged_algo_scores = shift.merge_algo_scores();
            sort_algo_scores_for_display(&mut merged_algo_scores, &mut algo_order, shift_index);

            for merged_algo_score in merged_algo_scores.iter() {
                let algo_name_raw = format!("{}", merged_algo_score.weighted_algo.algo);
                let algo_name = pad_to_width(&algo_name_raw, ALGO_COL_WIDTH);

                let normalization = format!(
                    "{:.3}",
                    merged_algo_score.weighted_algo.algo.normalization_factor()
                );
                let weight = format!("{:.3}", merged_algo_score.weighted_algo.weight);
                let average_score = format!("{:.3}", merged_algo_score.average_score);

                let scores = render_score_list_if_unequal(
                    &merged_algo_score.all_scores,
                    merged_algo_score.average_score,
                );

                println!(
                    "    {}  {:>norm_w$}  {:>weight_w$}  {:>total_w$}  {}",
                    algo_name,
                    normalization,
                    weight,
                    average_score,
                    scores,
                    norm_w = NORM_COL_WIDTH,
                    weight_w = WEIGHT_COL_WIDTH,
                    total_w = AVERAGE_COL_WIDTH,
                );
            }
        }
    }
    println!("");
}

fn sort_algo_scores_for_display(
    merged_algo_scores: &mut Vec<MergedAlgoScores>,
    algo_order: &mut Vec<*const WeightedAlgo>,
    shift_index: usize,
) {
    if shift_index == 0 {
        merged_algo_scores.sort_by(|a, b| b.average_score.partial_cmp(&a.average_score).unwrap());
        *algo_order = merged_algo_scores
            .iter()
            .map(|algo_score| algo_score.weighted_algo as *const WeightedAlgo)
            .collect();
        return;
    }

    merged_algo_scores.sort_by(|a, b| {
        let a_ptr = a.weighted_algo as *const WeightedAlgo;
        let b_ptr = b.weighted_algo as *const WeightedAlgo;
        let a_key = algo_order.iter().position(|ptr| *ptr == a_ptr).unwrap();
        let b_key = algo_order.iter().position(|ptr| *ptr == b_ptr).unwrap();
        a_key.cmp(&b_key)
    });
}

fn render_score_list_if_unequal(all_scores: &Vec<f64>, average_score: f64) -> String {
    let threshold_25 = (average_score * 0.25).abs();
    let threshold_50 = (average_score * 0.50).abs();

    let formatted_scores: Vec<_> = all_scores
        .iter()
        .map(|score| {
            let formatted = fmt_f64(score);

            let diff = (score - average_score).abs();

            if diff > threshold_50 {
                formatted.red().to_string()
            } else if diff > threshold_25 {
                formatted.yellow().to_string()
            } else {
                formatted.green().to_string()
            }
        })
        .collect();

    format!("({})", formatted_scores.join(","))
}

fn pad_to_width(value: &str, width: usize) -> String {
    let mut out = value.to_string();
    let used = UnicodeWidthStr::width(value);
    if used < width {
        out.push_str(&" ".repeat(width - used));
    }
    out
}

// strip trailing 0s and then a trailing . if that's all that's left
pub fn fmt_f64(val: &f64) -> String {
    format!("{:.3}", val)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn print_algo_summary(algo_summary_data: &[Vec<f64>], weighted_algos: &[WeightedAlgo]) {
    const ALGO_COL_WIDTH: usize = 27;
    const NUM_COL_WIDTH: usize = 10;

    println!("Algo score summary:");

    let algo_header = pad_to_width("Algo", ALGO_COL_WIDTH);
    let min_header = format!("{:>width$}", "Min", width = NUM_COL_WIDTH);
    let max_header = format!("{:>width$}", "Max", width = NUM_COL_WIDTH);
    let norm_extreme_header = format!("{:>width$}", "Norm(ext)", width = NUM_COL_WIDTH);
    let avg_header = format!("{:>width$}", "Avg", width = NUM_COL_WIDTH);
    let norm_avg_header = format!("{:>width$}", "Norm(avg)", width = NUM_COL_WIDTH);
    let median_header = format!("{:>width$}", "Median", width = NUM_COL_WIDTH);
    println!(
        "  {}{}{}{}{}{}{}",
        algo_header,
        min_header,
        max_header,
        norm_extreme_header,
        avg_header,
        norm_avg_header,
        median_header,
    );

    for (index, stats) in algo_summary_data.iter().enumerate() {
        let algo_name = pad_to_width(&format!("{}", weighted_algos[index].algo), ALGO_COL_WIDTH);

        let (min, max, avg, median) = summarize_values(stats);

        let normalization_factor = weighted_algos[index].algo.normalization_factor();
        let extreme = if min < 0.0 { min } else { max };
        let normalized_extreme = extreme * normalization_factor;
        let normalized_avg = avg * normalization_factor;

        let min = format!("{:>width$}", fmt_f64(&min), width = NUM_COL_WIDTH);
        let max = format!("{:>width$}", fmt_f64(&max), width = NUM_COL_WIDTH);
        let norm_extreme = format!(
            "{:>width$}",
            fmt_f64(&normalized_extreme),
            width = NUM_COL_WIDTH
        );
        let avg = format!("{:>width$}", fmt_f64(&avg), width = NUM_COL_WIDTH);
        let norm_avg = format!(
            "{:>width$}",
            fmt_f64(&normalized_avg),
            width = NUM_COL_WIDTH
        );
        let median = format!("{:>width$}", fmt_f64(&median), width = NUM_COL_WIDTH);

        println!(
            "  {}{}{}{}{}{}{}",
            algo_name, min, max, norm_extreme, avg, norm_avg, median,
        );
    }
    println!();
}

fn summarize_values(values: &[f64]) -> (f64, f64, f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for &value in values {
        min = min.min(value);
        max = max.max(value);
        sum += value;
    }

    let avg = sum / values.len() as f64;

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    let median = if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    };

    (min, max, avg, median)
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

        let dir = choose_direction(
            &game_state,
            &weighted_algos,
            1,
            &InsertionPoints::One,
            &mut rng,
            false,
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
                true,
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
        let dir = choose_direction(&game_state, &algos, 1, &InsertionPoints::One, &mut rng, false, &mut vec![]);
        assert_eq!(Direction::Right, dir, "With Empties strong, the best direction was right");

        // now swap the weights
        let algos: Vec<WeightedAlgo> = vec![
            WeightedAlgo { algo: Algos::Empties.to_algo(), weight: 1.0 },
            WeightedAlgo { algo: Algos::Merges.to_algo(), weight: 100.0 },
        ];
        let dir = choose_direction(&game_state, &algos, 1, &InsertionPoints::One, &mut rng, false, &mut vec![]);
        assert_eq!(Direction::Left, dir, "With Merges strong, the best direction was left");
    }
}
