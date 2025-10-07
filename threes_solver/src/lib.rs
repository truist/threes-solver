mod algo;
mod solver;

use std::collections::BTreeMap;
use std::time::Instant;

use rand::rngs::ThreadRng;
use rand::thread_rng;
use strum::IntoEnumIterator;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::Algos;

const GAMES: usize = 10_000;

struct PlayScore {
    moves: usize,
    high_card: Card,
}

pub fn solve() {
    let mut rng = thread_rng();
    let algos: Vec<Algos> = Algos::iter().collect();
    println!("Running with {} algos", algos.len());

    let mut results: Vec<PlayScore> = vec![];

    let start = Instant::now();

    for _ in 0..GAMES {
        let (moves, mut final_state) = play_game(&algos, &mut rng);
        let high_card = *final_state.high_card();

        results.push(PlayScore { moves, high_card });
    }

    let duration = start.elapsed();

    let total_moves = results
        .iter()
        .map(|play_score| play_score.moves)
        .sum::<usize>();

    let high_card_total = results
        .iter()
        .map(|play_score| play_score.high_card as usize)
        .sum::<usize>();

    println!(
        "Performed {} moves across {} games in {:?}",
        total_moves, GAMES, duration,
    );
    println!("Time per move: {:?}", duration.div_f64(total_moves as f64));
    println!("Average moves per game: {}", total_moves / GAMES);
    println!(
        "Average high card: {}",
        high_card_total as f64 / GAMES as f64
    );

    let mut counts: BTreeMap<Card, usize> = BTreeMap::new();
    for result in results {
        *counts.entry(result.high_card).or_insert(0) += 1;
    }
    for (card, count) in counts {
        println!("{card:?}: {count}");
    }
}

fn play_game(algos: &Vec<Algos>, rng: &mut ThreadRng) -> (usize, GameState) {
    solver::play(GameState::initialize(rng), algos, rng)
}
