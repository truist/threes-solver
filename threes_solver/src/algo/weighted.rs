use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

#[derive(Debug)]
pub struct WeightedAlgo {
    pub algo: Box<dyn Algo>,
    pub weight: f64,
}

impl WeightedAlgo {
    pub fn score(&self, game_state: &GameState) -> f64 {
        let base_score = self.algo.score(game_state, None);
        base_score * self.weight * self.algo.normalization_factor()
    }
}

impl fmt::Display for WeightedAlgo {
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
