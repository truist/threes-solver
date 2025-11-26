use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

#[derive(Debug)]
pub struct WeightedAlgo<A: ?Sized> {
    pub algo: Box<A>,
    pub weight: f64,
}

impl<A: Algo + ?Sized> WeightedAlgo<A> {
    pub fn score(&self, game_state: &Option<GameState>) -> f64 {
        let score = self.algo.score(game_state);
        assert!(
            // this 10 is a hack to get around scaled HighWall pushing the limits
            score <= i8::MAX / 2 + 10,
            "{:?} is getting dangerously close to the i8 score limit (got score of {score})\n{game_state:#?}",
            self.algo
        );
        score as f64 * self.weight
    }
}
