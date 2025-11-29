use std::fmt;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::Algos::Empties;
use crate::algo::core::{Algo, ValueFilter};

const MOVES_WHEN_WELL_INTO_GAME: f64 = 50.0;
const SCALE_MAX: f64 = 2.0;

#[derive(Debug)]
pub(crate) struct MovesScaler<A> {
    pub(crate) wrapped: A,
    pub(crate) positive: bool,
}

impl<A: Algo> MovesScaler<A> {
    // see the graph here: https://www.desmos.com/calculator/azrnzasjtw
    // (0 <= x <= 1)
    fn scale_score(&self, moves: usize, base_score: f64) -> f64 {
        let scaled_moves = (moves as f64 / MOVES_WHEN_WELL_INTO_GAME).min(1.0);

        let mut scale = SCALE_MAX
            * scaled_moves.powf(3.0)
            * (10.0 - 15.0 * scaled_moves + 6.0 * scaled_moves.powf(2.0));

        if !self.positive {
            scale = SCALE_MAX - scale;
        }

        base_score * scale
    }
}

impl<A: Algo> Algo for MovesScaler<A> {
    fn score(&self, game_state: &Option<GameState>, value_filter: Option<&dyn ValueFilter>) -> f64 {
        if let Some(actual_game_state) = game_state {
            let base_score = self.wrapped.score(game_state, value_filter);
            self.scale_score(actual_game_state.get_moves(), base_score)
        } else {
            0.0
        }
    }
}

impl<A: Algo> fmt::Display for MovesScaler<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = if self.positive { "📈" } else { "📉" };
        write!(f, "{} ({})", self.wrapped, dir)
    }
}

const EMPTIES_THRESHOLD: f64 = 5.0;

#[derive(Debug)]
pub(crate) struct FewEmptiesScaler<A> {
    pub(crate) wrapped: A,
}

impl<A: Algo> FewEmptiesScaler<A> {
    fn scale_score(&self, empties: f64, base_score: f64) -> f64 {
        // 6 empties -> 0 factor -> 0% boost -> 100%
        // 5 empties -> 1 factor -> 17% boost -> 117%
        // 4 empties -> 2 factor -> 33% boost -> 133%
        // 3 empties -> 3 factor -> 50% boost -> 150%
        // 2 empties -> 4 factor -> 67% boost -> 167%
        // 1 empties -> 5 factor -> 83% boost -> 183%
        // 0 empties -> 6 factor -> 100% boost -> 200%
        let factor = (EMPTIES_THRESHOLD + 1.0 - empties).max(0.0);
        let boost = factor / (EMPTIES_THRESHOLD + 1.0) * (SCALE_MAX - 1.0);
        let percentage = 1.0 + boost;
        base_score * percentage
    }
}

impl<A: Algo> Algo for FewEmptiesScaler<A> {
    fn score(&self, game_state: &Option<GameState>, value_filter: Option<&dyn ValueFilter>) -> f64 {
        if let Some(actual_game_state) = game_state {
            let base_score = self.wrapped.score(game_state, value_filter);
            let empties = Empties.empties(actual_game_state, None);

            self.scale_score(empties, base_score)
        } else {
            0.0
        }
    }
}

impl<A: Algo> fmt::Display for FewEmptiesScaler<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (scaled w/empties)", self.wrapped)
    }
}

#[derive(Debug)]
pub(crate) struct ValueFilterWrapper<A> {
    pub(crate) wrapped: A,
    pub(crate) min_value_to_keep: Card,
    pub(crate) max_value_to_keep: Card,
}

impl<A: Algo> Algo for ValueFilterWrapper<A> {
    fn score(&self, game_state: &Option<GameState>, value_filter: Option<&dyn ValueFilter>) -> f64 {
        if value_filter.is_some() {
            panic!("value_filter should always be unset in ValueFilterWrapper: {value_filter:?}")
        }

        self.wrapped.score(game_state, Some(self))
    }
}

impl<A: Algo> ValueFilter for ValueFilterWrapper<A> {
    fn filter_values(&self, values: &[Card]) -> bool {
        values
            .iter()
            .any(|val| self.min_value_to_keep <= *val && *val <= self.max_value_to_keep)
    }
}

impl<A: Algo> fmt::Display for ValueFilterWrapper<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (filtering {}-{})",
            self.wrapped, self.min_value_to_keep, self.max_value_to_keep,
        )
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::Algos;

    use super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    fn test_moves_scale_score() {
        let scale_positive = MovesScaler {
            wrapped: Empties, // doesn't matter what this is
            positive: true,
        };

        let wrapped_score = 7.0;

        assert_eq!(
            0.0,
            scale_positive.scale_score(0, wrapped_score),
            "at 0 moves, the scale is 0"
        );

        assert_eq!(
            SCALE_MAX * wrapped_score,
            scale_positive.scale_score(MOVES_WHEN_WELL_INTO_GAME as usize, wrapped_score),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            wrapped_score,
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize, wrapped_score),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert!(
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize, wrapped_score)
                > wrapped_score + 2.0,
            "at 60% into the core game, the scale is nudging the score up"
        );

        assert!(
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize, wrapped_score)
                < 2.0,
            "at 25% into the core game, the scale is still very low"
        );

        assert!(
            scale_positive.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize, wrapped_score)
                < 1.0,
            "at 10% into the core game, the scale is effectively 0"
        );

        let scale_negative = MovesScaler {
            wrapped: Algos::Empties, // doesn't matter what this is
            positive: false,
        };

        assert_eq!(
            SCALE_MAX * wrapped_score,
            scale_negative.scale_score(0, wrapped_score),
            "at 0 moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            0.0,
            scale_negative.scale_score(MOVES_WHEN_WELL_INTO_GAME as usize, wrapped_score),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is 0"
        );

        assert_eq!(
            wrapped_score,
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize, wrapped_score),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert!(
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize, wrapped_score)
                < wrapped_score - 2.0,
            "at 60% into the core game, the scale is nudging the score down"
        );

        assert!(
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize, wrapped_score)
                > 12.0,
            "at 25% into the core game, the scale is still very high"
        );

        assert!(
            scale_negative.scale_score((MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize, wrapped_score)
                > SCALE_MAX * wrapped_score - 1.0,
            "at 10% into the core game, the scale is effectively SCALE_MAX"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_empties_scale_score() {
        let empties_scaler = FewEmptiesScaler {
            wrapped: Algos::Empties, // doesn't matter what this is
        };

        let wrapped_score = 7.0;

        assert_eq!(
            wrapped_score,
            empties_scaler.scale_score(10.0, wrapped_score),
            "with many empties, the score is unmodified"
        );

        assert_eq!(
            wrapped_score * SCALE_MAX,
            empties_scaler.scale_score(0.0, wrapped_score),
            "with zero empties, the score is scaled as far as possible"
        );

        assert_eq!(
            wrapped_score * 1.5,
            empties_scaler.scale_score(3.0, wrapped_score),
            "with three empties, the score is scaled 50%"
        );

        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 3, 3, 3,
            3, 3, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(3.0, Empties.empties(&game_state, None), "empties() does what we think it does")
    }

    #[test]
    fn test_filter() {
        let wrapper = ValueFilterWrapper {
            wrapped: Algos::Merges,
            min_value_to_keep: 3,
            max_value_to_keep: 6,
        };

        assert!(
            !wrapper.filter_values(&[1]),
            "non-kept values are filtered out"
        );
        assert!(wrapper.filter_values(&[3]), "kept values are kept");
        assert!(
            !wrapper.filter_values(&[7]),
            "non-kept values are filtered out"
        );
        assert!(wrapper.filter_values(&[6]), "kept values are kept");
        assert!(
            !wrapper.filter_values(&[9]),
            "non-kept values are filtered out"
        );

        assert!(
            !wrapper.filter_values(&[1, 7, 9]),
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
