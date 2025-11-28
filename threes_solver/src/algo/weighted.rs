use threes_simulator::game_state::GameState;

use crate::algo::core::{Algo, AlgoScalers};

#[derive(Debug)]
pub struct WeightedAlgo<'a, A: ?Sized> {
    pub algo: Box<A>,
    pub weight: f64,
    pub scalers: AlgoScalers<'a>,
}

impl<'a, A: Algo + ?Sized> WeightedAlgo<'a, A> {
    fn score(&self, game_state: &Option<GameState>) -> f64 {
        self.algo.score(game_state, &self.scalers) * self.weight
    }
}
