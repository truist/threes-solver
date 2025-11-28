// TODO: rename this file

use std::fmt;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::AlgoScaler;

const MOVES_WHEN_WELL_INTO_GAME: f64 = 50.0;
const SCALE_MAX: f64 = 2.0;

#[derive(Debug)]
pub(crate) struct MovesScaler {
    pub(crate) positive: bool,
}

impl MovesScaler {
    // see the graph here: https://www.desmos.com/calculator/azrnzasjtw
    // (0 <= x <= 1)
    fn get_scale_for(&self, moves: usize) -> f64 {
        let scaled_moves = (moves as f64 / MOVES_WHEN_WELL_INTO_GAME).min(1.0);

        let mut scale = SCALE_MAX
            * scaled_moves.powf(3.0)
            * (10.0 - 15.0 * scaled_moves + 6.0 * scaled_moves.powf(2.0));

        if !self.positive {
            scale = SCALE_MAX - scale;
        }

        scale
    }
}

impl AlgoScaler for MovesScaler {
    fn scale_for(&self, game_state: &GameState, _: &[Card]) -> f64 {
        self.get_scale_for(game_state.get_moves())
    }
}

impl fmt::Display for MovesScaler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = if self.positive { "📈" } else { "📉" };
        write!(f, "{}", dir)
    }
}

#[derive(Debug)]
pub(crate) struct ValueScaler {
    pub(crate) min_value_to_scale: Card,
    pub(crate) max_value_to_scale: Card,
    pub(crate) scale: f64,
}

impl AlgoScaler for ValueScaler {
    fn scale_for(&self, _: &GameState, values: &[Card]) -> f64 {
        for val in values.iter() {
            if self.min_value_to_scale <= *val && *val <= self.max_value_to_scale {
                return self.scale;
            }
        }
        1.0
    }
}

impl fmt::Display for ValueScaler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scaling {}-{} by {}",
            self.min_value_to_scale, self.max_value_to_scale, self.scale,
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
    fn test_scale_score() {
        let scale_positive = MovesScaler { positive: true };

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

        let scale_negative = MovesScaler { positive: false };

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
    fn test_filter() {
        let game_state = generate_game_state([3; 16]);

        let test_scale = 3.0;
        let wrapper = ValueScaler {
            min_value_to_scale: 3,
            max_value_to_scale: 6,
            scale: test_scale,
        };

        assert_eq!(
            1.0,
            wrapper.scale_for(&game_state, &[1]),
            "non-matched values get no scaling"
        );
        assert_eq!(
            test_scale,
            wrapper.scale_for(&game_state, &[3]),
            "matched values are scaled"
        );
        assert_eq!(
            1.0,
            wrapper.scale_for(&game_state, &[9]),
            "non-matched values get no scaling"
        );

        assert_eq!(
            1.0,
            wrapper.scale_for(&game_state, &[1, 7, 9]),
            "if none of the values are matched, there is no scaling"
        );
        assert_eq!(
            test_scale,
            wrapper.scale_for(&game_state, &[3, 6]),
            "if all of the values are matched, the scale value is returned"
        );
        assert_eq!(
            test_scale,
            wrapper.scale_for(&game_state, &[1, 3, 5, 6, 9]),
            "if some of the values are matched, the scale value is returned"
        );
        assert_eq!(
            1.0,
            wrapper.scale_for(&game_state, &[1, 1, 1]),
            "duplicates are fine for non-matching"
        );
        assert_eq!(
            test_scale,
            wrapper.scale_for(&game_state, &[3, 3]),
            "duplicates are fine for matching"
        );
        assert_eq!(
            test_scale,
            wrapper.scale_for(&game_state, &[1, 1, 3, 3]),
            "duplicates are fine for matching"
        );
    }
}
