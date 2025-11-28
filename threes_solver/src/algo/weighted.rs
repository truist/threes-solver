use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::{Algo, ValueFilter};

#[derive(Debug)]
pub struct WeightedAlgo<A: ?Sized> {
    pub algo: Box<A>,
    pub weight: f64,
}

impl<A: Algo + ?Sized> Algo for WeightedAlgo<A> {
    fn score(&self, game_state: &Option<GameState>, value_filter: Option<&dyn ValueFilter>) -> f64 {
        self.algo.score(game_state, value_filter) * self.weight
    }
}

impl<A: Algo + ?Sized> fmt::Display for WeightedAlgo<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (weighted {})", self.algo, self.weight,)
    }
}
