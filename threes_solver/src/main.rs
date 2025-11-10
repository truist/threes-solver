mod algo;
mod optimizer;
mod solver;

use crate::algo::{Algos, WeightedAlgo};
use rng_util::RngType;
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use std::collections::BTreeMap;
use std::time::Instant;

use clap::{Parser, Subcommand};
use strum::{EnumCount, IntoEnumIterator};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    seed: Option<u64>,

    #[arg(long)]
    profiling: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Optional subcommand to run a single game, showing each step
    Simulate {
        /// optional list of all the algo weights (f64)
        #[arg(long, num_args = Algos::COUNT, value_name = "w")]
        weights: Option<Vec<f64>>,
    },
}

fn main() {
    let args = Args::parse();
    let (rng, seed) = rng_util::initialize_rng(args.seed);

    if let Some(Commands::Simulate { weights }) = args.command {
        simulate(rng, weights);
    } else {
        optimize(rng, seed, args.profiling);
    }
}

fn simulate(mut rng: RngType, weights: Option<Vec<f64>>) {
    let algos: Vec<WeightedAlgo> = if let Some(weights) = weights {
        Algos::iter()
            .zip(weights.iter())
            .map(|(algo, &weight)| WeightedAlgo { algo, weight })
            .collect()
    } else {
        Algos::iter()
            .map(|algo| WeightedAlgo { algo, weight: 1.0 })
            .collect()
    };

    let (_moves, final_state) = solver::play(GameState::initialize(&mut rng), &algos, &mut rng);
    println!("{final_state}");
}

fn optimize(mut rng: RngType, seed: u64, profiling: bool) {
    let start = Instant::now();
    let optimal_weights = optimizer::find_optimal_weights(&mut rng, seed, profiling);
    let duration = start.elapsed();
    println!("Optimizer ran for {duration:?}");

    let algos: Vec<WeightedAlgo> = Algos::iter()
        .zip(optimal_weights.final_mean.iter())
        .map(|(algo, &weight)| {
            println!("{:?}: {}", algo, weight);
            WeightedAlgo { algo, weight }
        })
        .collect();

    let mut high_cards: Vec<Card> = vec![];
    for _ in 0..optimizer::GAMES_PER_TEST {
        let (_moves, final_state) = solver::play(GameState::initialize(&mut rng), &algos, &mut rng);
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
