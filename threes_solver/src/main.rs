mod algo;
mod optimizer;
mod solver;

use crate::algo::{Algos, WeightedAlgo};
use rng_util::RngType;
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use std::collections::BTreeMap;
use std::time::Instant;

use strum::IntoEnumIterator;

fn parse_args() -> (RngType, u64, bool) {
    let mut args = std::env::args().skip(1);

    let mut seed: Option<String> = None;
    let mut profiling = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                if let Some(val) = args.next() {
                    seed = Some(val);
                } else {
                    eprintln!("error: --seed requires a value");
                    std::process::exit(1);
                }
            }
            "--profiling" => profiling = true,
            _ => {
                eprintln!("unknown argument: {arg}");
                eprintln!("args: [--seed <hex string>] [--profiling]");
                std::process::exit(1);
            }
        }
    }

    let (rng, seed) = rng_util::initialize_rng(seed);
    (rng, seed, profiling)
}

fn main() {
    let (mut rng, seed, profiling) = parse_args();

    let start = Instant::now();
    let optimal_weights = optimizer::find_optimal_weights(&mut rng, seed, profiling);
    let duration = start.elapsed();
    println!("Ran for {duration:?}");

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
