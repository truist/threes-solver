mod algo;
mod optimizer;
mod solver;

use crate::algo::{Algos, WeightedAlgo};
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use std::collections::BTreeMap;
use std::time::Instant;

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use strum::IntoEnumIterator;

fn main() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);

    let start = Instant::now();
    let optimal_weights = optimizer::find_optimal_weights(&mut rng);
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
