use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

#[derive(Debug)]
pub struct WeightedAlgo<A: ?Sized> {
    pub algo: Box<A>,
    pub weight: f64,
}

impl<A: Algo + ?Sized> WeightedAlgo<A> {
    pub fn score(&self, game_state: &GameState) -> f64 {
        let base_score = self.algo.score(game_state, None);
        base_score * self.weight * self.algo.normalization_factor()
    }
}

impl<A: Algo + ?Sized> fmt::Display for WeightedAlgo<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (normalized by {}, weighted {})",
            self.algo,
            self.algo.normalization_factor(),
            self.weight,
        )
    }
}
