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
        /// Where to write the weights TOML file
        #[arg(long, default_value = DEFAULT_WEIGHTS_FILE_NAME)]
        weights_file: PathBuf,

        /// Loosen the tolerances and stop earlier
        #[arg(long)]
        rough: bool,
    },

    /// Optional subcommand to run a single game, showing each step
    Simulate {
        /// Read weights from this TOML file
        #[arg(long, default_value = DEFAULT_WEIGHTS_FILE_NAME)]
        weights_file: PathBuf,

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
        Some(Commands::Simulate {
            weights_file,
            batch,
        }) => simulate(rng, weights_file, batch),

        Some(Commands::Optimize {
            weights_file,
            rough,
        }) => optimize(rng, seed, args.profiling, rough, weights_file),

        None => optimize(
            rng,
            seed,
            false,
            args.profiling,
            PathBuf::from(DEFAULT_WEIGHTS_FILE_NAME),
        ),
    }
}

fn simulate(mut rng: RngType, weights_file: PathBuf, batch: bool) {
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
        run_batch(rng, weighted_algos);
    } else {
        solver::play(
            GameState::initialize(&mut rng),
            &weighted_algos,
            &mut rng,
            true,
        );
    }
}

fn optimize(mut rng: RngType, seed: u64, profiling: bool, rough: bool, weights_file: PathBuf) {
    let start = Instant::now();
    let optimal_weights = optimizer::find_optimal_weights(&mut rng, seed, profiling, rough);
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

    run_batch(rng, algos);
}

fn run_batch(mut rng: RngType, weighted_algos: Vec<WeightedAlgo<dyn Algo>>) {
    let mut high_cards: Vec<Card> = vec![];
    for _ in 0..optimizer::GAMES_PER_TEST {
        let (_moves, final_state) = solver::play(
            GameState::initialize(&mut rng),
            &weighted_algos,
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
