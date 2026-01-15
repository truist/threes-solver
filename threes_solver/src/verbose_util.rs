use std::fmt;
use std::ops::{Deref, DerefMut};

use crossterm::style::Stylize;
use unicode_width::UnicodeWidthStr;

use crate::solver::Shift;
use crate::WeightedAlgo;

pub(crate) enum Verbose {
    Off,
    Details,
    Insights,
}

// strip trailing 0s and then a trailing . if that's all that's left
pub(crate) fn fmt_f64(val: &f64) -> String {
    format!("{:.3}", val)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub(crate) fn print_verbose(verbose: &Verbose, shifts: &mut Vec<Shift>) {
    if matches!(verbose, Verbose::Off) {
        return;
    }

    println!("Considered these shifts:");
    if let Verbose::Details = verbose {
        print_detail_column_headers();
    }

    shifts.reverse();

    let all_shifts_merged = merge_algo_scores_for_all_shifts(shifts);
    for shift_index in 0..all_shifts_merged.len() {
        let shift_merged = &all_shifts_merged[shift_index];
        let shift = shift_merged.shift;
        if !shift.could_shift {
            println!("  {} (can't)", shift.direction);
        } else {
            println!("  {} ({}): ", shift.direction, fmt_f64(&shift.total_score));

            match verbose {
                Verbose::Details => {
                    print_detail_columns(&shift_merged.merged_algo_scores_list.as_ref().unwrap())
                }
                Verbose::Insights => print_insights_for(&all_shifts_merged, shift_index),
                Verbose::Off => panic!("It shouldn't be possible to get to this line of code"),
            }
        }
    }
    println!("");
}

fn print_insights_for(all_shifts_merged: &Vec<ShiftWithMergedScores>, shift_index: usize) {
    let shift_with_merged_scores = &all_shifts_merged[shift_index];
    let current = &shift_with_merged_scores
        .merged_algo_scores_list
        .as_ref()
        .unwrap();
    let current_total = shift_with_merged_scores.shift.total_score;

    let mut next = &None;
    for next_index in shift_index + 1..all_shifts_merged.len() {
        let next_shift_merged = &all_shifts_merged[next_index];
        if next_shift_merged.shift.could_shift {
            next = &next_shift_merged.merged_algo_scores_list;
            break;
        }
    }

    let mut algo_diffs = vec![];
    for (algo_index, current_merged_algo_scores) in current.iter().enumerate() {
        let next_score = match next {
            Some(next) => next.merged_algo_scores[algo_index].average_score,
            None => 0.0,
        };
        algo_diffs.push(AlgoScoreDiff::new(
            current_merged_algo_scores.weighted_algo,
            current_merged_algo_scores.average_score,
            next_score,
            current_total,
            &current_merged_algo_scores.all_scores,
        ));
    }

    algo_diffs.sort_by(|a, b| b.score_diff.partial_cmp(&a.score_diff).unwrap());
    for (algo_index, algo_diff) in algo_diffs.iter().enumerate() {
        if (shift_index == 0 && (algo_index < 2 || algo_index >= algo_diffs.len() - 2))
            || algo_diff.is_big
        {
            println!("    {algo_diff}");
        }
    }
}

#[derive(Debug)]
struct AlgoScoreDiff<'a> {
    weighted_algo: &'a WeightedAlgo,

    current_score: f64,
    score_diff: f64,

    is_big: bool,

    current_raw: &'a Vec<f64>,
}

impl<'a> AlgoScoreDiff<'a> {
    fn new(
        weighted_algo: &'a WeightedAlgo,
        current_score: f64,
        next_score: f64,
        current_total: f64,
        current_raw: &'a Vec<f64>,
    ) -> AlgoScoreDiff<'a> {
        AlgoScoreDiff {
            weighted_algo,
            current_score,
            score_diff: current_score - next_score,
            is_big: current_score.abs() > (current_total * 0.25).abs(),
            current_raw,
        }
    }
}

impl<'a> fmt::Display for AlgoScoreDiff<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let score_list = if self.current_raw.windows(2).all(|w| w[0] == w[1]) {
            "".to_string()
        } else {
            format!(
                " ({})",
                render_score_list(self.current_raw, self.current_score)
            )
        };
        write!(
            f,
            "{}: {} / {}{}",
            self.weighted_algo.algo,
            fmt_f64(&self.score_diff),
            fmt_f64(&self.current_score),
            score_list,
        )
    }
}

struct MergedAlgoScores<'a> {
    weighted_algo: &'a WeightedAlgo,
    average_score: f64,
    all_scores: Vec<f64>,
}

struct MergedAlgoScoresList<'a> {
    merged_algo_scores: Vec<MergedAlgoScores<'a>>,
}
// Make refs to MergedAlgoScoresList behave like refs to merged_algo_scores
impl<'a> Deref for MergedAlgoScoresList<'a> {
    type Target = Vec<MergedAlgoScores<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.merged_algo_scores
    }
}
impl<'a> DerefMut for MergedAlgoScoresList<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.merged_algo_scores
    }
}

struct ShiftWithMergedScores<'a> {
    shift: &'a Shift<'a>,
    merged_algo_scores_list: Option<MergedAlgoScoresList<'a>>,
}

fn merge_algo_scores_for_all_shifts<'a>(shifts: &'a Vec<Shift>) -> Vec<ShiftWithMergedScores<'a>> {
    let mut all_shifts_merged = vec![];

    let mut algo_order: Vec<*const WeightedAlgo> = Vec::new();
    for (shift_index, shift) in shifts.iter().enumerate() {
        let merged_algo_scores_list = if shift.could_shift {
            let mut merged_algo_scores_list = merge_algo_scores(shift);
            sort_algo_scores_for_display(
                &mut merged_algo_scores_list,
                &mut algo_order,
                shift_index,
            );
            Some(merged_algo_scores_list)
        } else {
            None
        };

        all_shifts_merged.push(ShiftWithMergedScores {
            shift,
            merged_algo_scores_list,
        });
    }

    all_shifts_merged
}

fn merge_algo_scores<'a>(shift: &'a Shift) -> MergedAlgoScoresList<'a> {
    let mut merged_algo_scores_list = MergedAlgoScoresList {
        merged_algo_scores: vec![],
    };
    if shift.state_scores.len() == 0 {
        return merged_algo_scores_list;
    }

    let first_algo_scores = &shift.state_scores.first().unwrap().algo_scores;
    for wai in 0..first_algo_scores.len() {
        let all_scores: Vec<f64> = shift
            .state_scores
            .iter()
            .map(|state_score| state_score.algo_scores[wai].score)
            .collect();
        let average_score = all_scores.iter().sum::<f64>() / all_scores.len().max(1) as f64;

        let weighted_algo = first_algo_scores[wai].weighted_algo;

        merged_algo_scores_list.push(MergedAlgoScores {
            weighted_algo,
            average_score,
            all_scores,
        });
    }

    merged_algo_scores_list
}

fn sort_algo_scores_for_display(
    merged_algo_scores_list: &mut MergedAlgoScoresList,
    algo_order: &mut Vec<*const WeightedAlgo>,
    shift_index: usize,
) {
    if shift_index == 0 {
        merged_algo_scores_list
            .sort_by(|a, b| b.average_score.partial_cmp(&a.average_score).unwrap());
        *algo_order = merged_algo_scores_list
            .iter()
            .map(|algo_score| algo_score.weighted_algo as *const WeightedAlgo)
            .collect();
        return;
    }

    merged_algo_scores_list.sort_by(|a, b| {
        let a_ptr = a.weighted_algo as *const WeightedAlgo;
        let b_ptr = b.weighted_algo as *const WeightedAlgo;
        let a_key = algo_order.iter().position(|ptr| *ptr == a_ptr).unwrap();
        let b_key = algo_order.iter().position(|ptr| *ptr == b_ptr).unwrap();
        a_key.cmp(&b_key)
    });
}

const ALGO_COL_WIDTH: usize = 32;
const NORM_COL_WIDTH: usize = 5;
const WEIGHT_COL_WIDTH: usize = 5;
const AVERAGE_COL_WIDTH: usize = 7;

fn print_detail_column_headers() {
    let algo_header = pad_to_width("Algo", ALGO_COL_WIDTH).blue();
    let norm_header = pad_to_width("Norm", NORM_COL_WIDTH).blue();
    let weight_header = pad_to_width("Weight", WEIGHT_COL_WIDTH).blue();
    let average_header = pad_to_width("Average", AVERAGE_COL_WIDTH).blue();
    let insertion_header = "Insertion variations".blue();
    println!(
        "    {}  {}  {}  {}  {}",
        algo_header, norm_header, weight_header, average_header, insertion_header,
    );
}

fn print_detail_columns(merged_algo_scores_list: &MergedAlgoScoresList) {
    for merged_algo_score in merged_algo_scores_list.iter() {
        let algo_name_raw = format!("{}", merged_algo_score.weighted_algo.algo);
        let algo_name = pad_to_width(&algo_name_raw, ALGO_COL_WIDTH);

        let normalization = format!(
            "{:.3}",
            merged_algo_score.weighted_algo.algo.normalization_factor()
        );
        let weight = format!("{:.3}", merged_algo_score.weighted_algo.weight);
        let average_score = format!("{:.3}", merged_algo_score.average_score);

        let scores = render_score_list(
            &merged_algo_score.all_scores,
            merged_algo_score.average_score,
        );

        println!(
            "    {}  {:>norm_w$}  {:>weight_w$}  {:>total_w$}  ({})",
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

fn render_score_list(all_scores: &Vec<f64>, average_score: f64) -> String {
    let threshold_25 = (average_score * 0.25).abs();
    let threshold_50 = (average_score * 0.50).abs();

    let mut sorted_scores = all_scores.clone();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let formatted_scores: Vec<_> = sorted_scores
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

    // format!("({})", formatted_scores.join(","))
    formatted_scores.join(",")
}

fn pad_to_width(value: &str, width: usize) -> String {
    let mut out = value.to_string();
    let used = UnicodeWidthStr::width(value);
    if used < width {
        out.push_str(&" ".repeat(width - used));
    }
    out
}

pub(crate) fn print_algo_summary(algo_summary_data: &[Vec<f64>], weighted_algos: &[WeightedAlgo]) {
    const ALGO_COL_WIDTH: usize = 32;
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
