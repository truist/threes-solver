use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use rng_util::RngType;
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use crate::algo::{Algo, WeightedAlgo};

mod algo;
mod optimizer;
mod solver;

const DEFAULT_WEIGHTS_FILE_NAME: &str = "weights.toml";

#[derive(Parser)]
struct Args {
    /// Set the seed for the RNG (u64)
    #[arg(long)]
    seed: Option<u64>,

    /// Path to read or write the weights TOML file
    #[arg(long, default_value = DEFAULT_WEIGHTS_FILE_NAME)]
    weights_file: PathBuf,

    /// Do NOT evaluate (and average) all possible next-card insertion points
    #[arg(long)]
    single_insertion_point: bool,

    /// Profiling mode (single thread, fewer generations)
    #[arg(long)]
    profiling: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Default subcommand to discover optimal weights
    Optimize {
        /// Loosen the tolerances and stop earlier
        #[arg(long)]
        rough: bool,
    },

    /// Optional subcommand to run a single game, showing each step
    Simulate {
        /// Simulate a batch of games and report the aggregate results
        #[arg(long)]
        batch: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct WeightsConfig {
    pub weights: Vec<f64>,
}

fn main() {
    let args = Args::parse();
    let (rng, seed) = rng_util::initialize_rng(args.seed);

    match args.command {
        Some(Commands::Simulate { batch }) => {
            simulate(rng, args.weights_file, !args.single_insertion_point, batch)
        }

        Some(Commands::Optimize { rough }) => optimize(
            rng,
            seed,
            args.weights_file,
            args.profiling,
            !args.single_insertion_point,
            rough,
        ),

        None => optimize(rng, seed, args.weights_file, args.profiling, true, false),
    }
}

fn simulate(mut rng: RngType, weights_file: PathBuf, all_insertion_points: bool, batch: bool) {
    let algos = crate::algo::build_all_algos();

    let weights_to_use = if weights_file.as_path().exists() {
        let toml_str = fs::read_to_string(weights_file).unwrap();
        let config: WeightsConfig = toml::from_str(&toml_str).unwrap();
        config.weights
    } else {
        eprintln!("Weights file ({weights_file:?}) doesn't exist; using default weights.");

        algos.iter().map(|_| 1.0).collect()
    };
    println!("Using weights: {weights_to_use:?}");

    let expected = algos.len();
    let actual = weights_to_use.len();
    if actual != expected {
        panic!("Incorrect number of weights supplied; expected {expected} but got {actual}");
    }

    let weighted_algos: Vec<WeightedAlgo<dyn Algo>> = algos
        .into_iter()
        .zip(weights_to_use.iter())
        .map(|(algo, &weight)| WeightedAlgo { algo, weight })
        .collect();

    if batch {
        run_batch(rng, weighted_algos, all_insertion_points);
    } else {
        solver::play(
            GameState::initialize(&mut rng),
            &weighted_algos,
            all_insertion_points,
            &mut rng,
            true,
        );
    }
}

fn optimize(
    mut rng: RngType,
    seed: u64,
    weights_file: PathBuf,
    profiling: bool,
    all_insertion_points: bool,
    rough: bool,
) {
    let start = Instant::now();
    let optimal_weights =
        optimizer::find_optimal_weights(&mut rng, seed, all_insertion_points, profiling, rough);
    let duration = start.elapsed();
    println!("Optimizer ran for {duration:?}");

    let mut toml_weights = vec![];
    let algos: Vec<WeightedAlgo<dyn Algo>> = crate::algo::build_all_algos()
        .into_iter()
        .zip(optimal_weights.final_mean.iter())
        .map(|(algo, &weight)| {
            println!("{}: {}", algo, weight);

            toml_weights.push(weight);

            WeightedAlgo { algo, weight }
        })
        .collect();

    let config = WeightsConfig {
        weights: toml_weights,
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    fs::write(weights_file, toml_str).unwrap();

    run_batch(rng, algos, all_insertion_points);
}

fn run_batch(
    mut rng: RngType,
    weighted_algos: Vec<WeightedAlgo<dyn Algo>>,
    all_insertion_points: bool,
) {
    let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
    println!(
        "Running batch of {} games, evaluating {} insertion point(s) per shift",
        optimizer::GAMES_PER_TEST,
        insertion_point_desc
    );
    let mut high_cards: Vec<Card> = vec![];
    for _ in 0..optimizer::GAMES_PER_TEST {
        let (_moves, final_state) = solver::play(
            GameState::initialize(&mut rng),
            &weighted_algos,
            all_insertion_points,
            &mut rng,
            false,
        );
        high_cards.push(*final_state.high_card());
    }

    let mut counts: BTreeMap<Card, usize> = BTreeMap::new();
    for high_card in high_cards {
        *counts.entry(high_card).or_insert(0) += 1;
    }
    for (card, count) in counts {
        println!("{card:?}: {count}");
    }
}
