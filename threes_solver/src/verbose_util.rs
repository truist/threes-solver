use crossterm::style::Stylize;
use unicode_width::UnicodeWidthStr;

use crate::solver::Shift;
use crate::WeightedAlgo;

pub(crate) enum Verbose {
    Off,
    Details,
    Insights,
}

struct MergedAlgoScores<'a> {
    weighted_algo: &'a WeightedAlgo,
    average_score: f64,
    all_scores: Vec<f64>,
}

fn merge_algo_scores<'a>(shift: &'a Shift) -> Vec<MergedAlgoScores<'a>> {
    let mut merged = vec![];
    if shift.state_scores.len() == 0 {
        return merged;
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

        merged.push(MergedAlgoScores {
            weighted_algo,
            average_score,
            all_scores,
        });
    }

    merged
}

pub(crate) fn print_verbose(verbose: &Verbose, shifts: &mut Vec<Shift>) {
    if matches!(verbose, Verbose::Off) {
        return;
    }

    const ALGO_COL_WIDTH: usize = 32;
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

            let mut merged_algo_scores = merge_algo_scores(shift);
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
pub(crate) fn fmt_f64(val: &f64) -> String {
    format!("{:.3}", val)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
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
