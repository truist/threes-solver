use std::fmt;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::{Algo, AlgoValueFilter};

const MOVES_WHEN_WELL_INTO_GAME: f64 = 50.0;
const SCALE_MAX: f64 = 2.0;

#[derive(Debug)]
pub(crate) struct MovesScaled<A> {
    pub(crate) wrapped: A,
    pub(crate) positive: bool,
}

impl<A: Algo> MovesScaled<A> {
    // see the graph here: https://www.desmos.com/calculator/azrnzasjtw
    // (0 <= x <= 1)
    fn scale_score(&self, moves: usize, base_score: i8) -> i8 {
        let scaled_moves = (moves as f64 / MOVES_WHEN_WELL_INTO_GAME).min(1.0);

        let mut scale = SCALE_MAX
            * scaled_moves.powf(3.0)
            * (10.0 - 15.0 * scaled_moves + 6.0 * scaled_moves.powf(2.0));

        if !self.positive {
            scale = SCALE_MAX - scale;
        }

        (base_score as f64 * scale).round() as i8
    }
}

impl<A: Algo> Algo for MovesScaled<A> {
    fn score(
        &self,
        game_state: &Option<GameState>,
        value_filter: Option<&dyn AlgoValueFilter>,
    ) -> i8 {
        if let Some(actual_game_state) = game_state {
            let base_score = self.wrapped.score(game_state, value_filter);
            self.scale_score(actual_game_state.get_moves(), base_score)
        } else {
            0
        }
    }
}

impl<A: Algo> fmt::Display for MovesScaled<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = if self.positive { "📈" } else { "📉" };
        write!(f, "{} ({})", self.wrapped, dir)
    }
}

#[derive(Debug)]
pub(crate) struct AlgoValueFilterWrapper<A> {
    pub(crate) wrapped: A,
    pub(crate) values_to_keep: Vec<Card>,
}

impl<A: Algo> Algo for AlgoValueFilterWrapper<A> {
    fn score(
        &self,
        game_state: &Option<GameState>,
        value_filter: Option<&dyn AlgoValueFilter>,
    ) -> i8 {
        if value_filter.is_some() {
            panic!(
                "value_filter should always be unset in AlgoValueFilterWrapper: {value_filter:?}"
            )
        }

        self.wrapped.score(game_state, Some(self))
    }
}

impl<A: Algo> AlgoValueFilter for AlgoValueFilterWrapper<A> {
    fn filter_values(&self, values: &[Card]) -> bool {
        values.iter().any(|val| self.values_to_keep.contains(&val))
    }
}

impl<A: Algo> fmt::Display for AlgoValueFilterWrapper<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (filtering {})",
            self.wrapped,
            self.values_to_keep
                .iter()
                .map(|val| val.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::Algos;

    use super::*;

    #[test]
    fn test_scale_score() {
        let scale_positive = MovesScaled {
            wrapped: Algos::Empties, // doesn't matter what this is
            positive: true,
        };

        let wrapped_score = 7;

        assert_eq!(
            0,
            scale_positive.scale_score(0, wrapped_score),
            "at 0 moves, the scale is 0"
        );

        assert_eq!(
            SCALE_MAX as i8 * wrapped_score,
            scale_positive.scale_score(MOVES_WHEN_WELL_INTO_GAME as usize, wrapped_score),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            wrapped_score,
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize, wrapped_score),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert_eq!(
            wrapped_score + 3,
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize, wrapped_score),
            "at 60% into the core game, the scale is nudging the score up"
        );

        assert_eq!(
            1,
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize, wrapped_score),
            "at 25% into the core game, the scale is still very low"
        );

        assert_eq!(
            0,
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize, wrapped_score),
            "at 10% into the core game, the scale is effectively 0"
        );

        let scale_negative = MovesScaled {
            wrapped: Algos::Empties, // doesn't matter what this is
            positive: false,
        };

        assert_eq!(
            SCALE_MAX as i8 * wrapped_score,
            scale_negative.scale_score(0, wrapped_score),
            "at 0 moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            0,
            scale_negative.scale_score(MOVES_WHEN_WELL_INTO_GAME as usize, wrapped_score),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is 0"
        );

        assert_eq!(
            wrapped_score,
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize, wrapped_score),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert_eq!(
            wrapped_score - 3,
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize, wrapped_score),
            "at 60% into the core game, the scale is nudging the score down"
        );

        assert_eq!(
            13,
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize, wrapped_score),
            "at 25% into the core game, the scale is still very high"
        );

        assert_eq!(
            SCALE_MAX as i8 * wrapped_score,
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize, wrapped_score),
            "at 10% into the core game, the scale is effectively SCALE_MAX"
        );
    }

    #[test]
    fn test_filter() {
        let wrapper = AlgoValueFilterWrapper {
            wrapped: Algos::Merges,
            values_to_keep: vec![3, 6],
        };

        assert!(
            !wrapper.filter_values(&[1]),
            "non-kept values are filtered out"
        );
        assert!(wrapper.filter_values(&[3]), "kept values are kept");
        assert!(
            !wrapper.filter_values(&[5]),
            "non-kept values are filtered out"
        );
        assert!(wrapper.filter_values(&[6]), "kept values are kept");
        assert!(
            !wrapper.filter_values(&[9]),
            "non-kept values are filtered out"
        );

        assert!(
            !wrapper.filter_values(&[1, 5, 9]),
            "if none of the values are kept, all are filtered out"
        );
        assert!(
            wrapper.filter_values(&[3, 6]),
            "if all of the values are kept, they are all kept"
        );
        assert!(
            wrapper.filter_values(&[1, 3, 5, 6, 9]),
            "if some of the values are kept, they are all kept"
        );
        assert!(
            !wrapper.filter_values(&[1, 1, 1]),
            "duplicates are fine for rejection"
        );
        assert!(
            wrapper.filter_values(&[3, 3]),
            "duplicates are fine for acceptance"
        );
        assert!(
            wrapper.filter_values(&[1, 1, 3, 3]),
            "duplicates are fine for mixed"
        );
    }
}
