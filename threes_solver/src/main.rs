mod algo;
mod optimizer;
mod solver;

use crate::algo::{Algos, WeightedAlgo};
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use std::collections::BTreeMap;

use rand::thread_rng;
use strum::IntoEnumIterator;

fn main() {
    let mut rng = thread_rng();

    let optimal_weights = optimizer::find_optimal_weights(&mut rng);

    let algos: Vec<WeightedAlgo> = Algos::iter()
        .zip(optimal_weights.final_mean.iter())
        .map(|(algo, &weight)| WeightedAlgo { algo, weight })
        .collect();

    let mut high_cards: Vec<Card> = vec![];
    for _ in 0..100 {
        let (_moves, mut final_state) =
            solver::play(GameState::initialize(&mut rng), &algos, &mut rng);
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
