use std::fmt;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::{Algo, ValueBooster};

#[derive(Debug)]
pub(crate) struct ValueBoosterWrapper<A> {
    pub(crate) wrapped: A,
    pub(crate) min_value_to_boost: Card,
    pub(crate) max_value_to_boost: Card,
    pub(crate) boost: f64,
}

impl<A: Algo> Algo for ValueBoosterWrapper<A> {
    fn score(&self, game_state: &GameState, value_booster: Option<&dyn ValueBooster>) -> f64 {
        assert!(
            value_booster.is_none(),
            "value_booster should always be unset in ValueBoosterWrapper: {value_booster:?}"
        );

        self.wrapped.score(game_state, Some(self))
    }

    fn normalization_factor(&self) -> f64 {
        //wrapped is supposed to be normalized by 2.0
        //  so the base max value is 12, which will become 24
        //we inherently boost by 2.0, so 12 will become 24
        //so we should normalize by 24 / 24 = 1.0
        //  normalize / 2 = 1
        //so 12 * 2 * 1 = 24
        //
        //wrapped is supposed to be normalized by 0.5, so 12 will become 6
        //  so the base max value is 48, which will become 24
        //we inherently boost by 2.0, so 48 will become 96
        //so we should normalize by 24 / 96 = 0.25
        //  normalize / boost = 0.25
        //so 48 * 2 * 0.25 = 24
        //
        //wrapped: 2.0
        //  base max: 12 -> 24
        //boost: 0.5, so 12 -> 6
        //so normalize by 24 / 6 = 4
        //  normalize / boost = 4
        //so 12 * 0.5 * 4 = 24
        //
        //wrapped: 0.5
        //  base max: 48 -> 24
        //boost: 0.5, so 48 -> 24
        //so normalize by 24 / 24 = 1
        //  normalize / boost = 1
        //so 48 * 0.5 * 1 = 24
        //
        self.wrapped.normalization_factor() / self.boost
    }
}

impl<A: Algo> ValueBooster for ValueBoosterWrapper<A> {
    fn boost_score_for(&self, score: f64, values: &[Card]) -> f64 {
        if values
            .iter()
            .any(|val| self.min_value_to_boost <= *val && *val <= self.max_value_to_boost)
        {
            score * self.boost
        } else {
            score
        }
    }
}

impl<A: Algo> fmt::Display for ValueBoosterWrapper<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (boosting {}-{} by {})",
            self.wrapped, self.min_value_to_boost, self.max_value_to_boost, self.boost,
        )
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::Algos;

    use super::*;

    #[test]
    fn test_value_booster() {
        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: Algos::Merges,
            min_value_to_boost: 3,
            max_value_to_boost: 6,
            boost: test_boost,
        };

        assert_eq!(
            1.0,
            booster.boost_score_for(1.0, &[1]),
            "non-boosted values aren't boosted"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[3]),
            "boosted values are boosted"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[6]),
            "boosted values are boosted"
        );
        assert_eq!(
            1.0,
            booster.boost_score_for(1.0, &[9]),
            "non-boosted values aren't boosted"
        );

        assert_eq!(
            1.0,
            booster.boost_score_for(1.0, &[1, 7, 9]),
            "if none of the values are boosted, the score isn't boosted"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[3, 6]),
            "if all of the values are boosted, the score is boosted"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[1, 3, 5, 6, 9]),
            "if some of the values are boosted, the score is boosted"
        );
        assert_eq!(
            1.0,
            booster.boost_score_for(1.0, &[1, 1, 1]),
            "duplicates are fine when not boosting"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[3, 3]),
            "duplicates are fine when boosting"
        );
        assert_eq!(
            test_boost,
            booster.boost_score_for(1.0, &[1, 1, 3, 3]),
            "duplicates are fine for mixed"
        );
    }
}
