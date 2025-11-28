use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

#[derive(Debug)]
pub struct WeightedAlgo<A: ?Sized> {
    pub algo: Box<A>,
    pub weight: f64,
}

impl<A: Algo + ?Sized> WeightedAlgo<A> {
    pub fn score(&self, game_state: &Option<GameState>) -> f64 {
        self.algo.score(game_state, None) * self.weight
    }
}
