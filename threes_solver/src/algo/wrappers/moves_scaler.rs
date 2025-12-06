use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::{Algo, ValueBooster};

#[derive(Debug)]
pub(crate) struct MovesScaler {
    pub(crate) wrapped: Box<dyn Algo>,
    pub(crate) positive: bool,
}

impl MovesScaler {
    const MOVES_WHEN_WELL_INTO_GAME: f64 = 50.0;
    const SCALE_MAX: f64 = 2.0;

    // see the graph here: https://www.desmos.com/calculator/azrnzasjtw
    // (0 <= x <= 1)
    fn scale_score(&self, moves: usize, base_score: f64) -> f64 {
        let scaled_moves = (moves as f64 / Self::MOVES_WHEN_WELL_INTO_GAME).min(1.0);

        let mut scale = Self::SCALE_MAX
            * scaled_moves.powf(3.0)
            * (10.0 - 15.0 * scaled_moves + 6.0 * scaled_moves.powf(2.0));

        if !self.positive {
            scale = Self::SCALE_MAX - scale;
        }

        base_score * scale
    }
}

impl Algo for MovesScaler {
    fn score(&self, game_state: &GameState, value_booster: Option<&dyn ValueBooster>) -> f64 {
        let base_score = self.wrapped.score(game_state, value_booster);
        self.scale_score(game_state.get_moves(), base_score)
    }

    fn normalization_factor(&self) -> f64 {
        // see comment in ValueBooster.normalization_factor()
        self.wrapped.normalization_factor() / Self::SCALE_MAX
    }
}

impl fmt::Display for MovesScaler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = if self.positive { "📈" } else { "📉" };
        write!(f, "{} ({})", self.wrapped, dir)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::impls::empties::Empties;

    use super::*;

    #[test]
    fn test_moves_scale_score() {
        let scale_positive = MovesScaler {
            wrapped: Box::new(Empties), // doesn't matter what this is
            positive: true,
        };

        let wrapped_score = 7.0;

        assert_eq!(
            0.0,
            scale_positive.scale_score(0, wrapped_score),
            "at 0 moves, the scale is 0"
        );

        assert_eq!(
            MovesScaler::SCALE_MAX * wrapped_score,
            scale_positive.scale_score(
                MovesScaler::MOVES_WHEN_WELL_INTO_GAME as usize,
                wrapped_score
            ),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            wrapped_score,
            scale_positive.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize,
                wrapped_score
            ),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert!(
            scale_positive.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize,
                wrapped_score
            ) > wrapped_score + 2.0,
            "at 60% into the core game, the scale is nudging the score up"
        );

        assert!(
            scale_positive.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize,
                wrapped_score
            ) < 2.0,
            "at 25% into the core game, the scale is still very low"
        );

        assert!(
            scale_positive.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize,
                wrapped_score
            ) < 1.0,
            "at 10% into the core game, the scale is effectively 0"
        );

        let scale_negative = MovesScaler {
            wrapped: Box::new(Empties), // doesn't matter what this is
            positive: false,
        };

        assert_eq!(
            MovesScaler::SCALE_MAX * wrapped_score,
            scale_negative.scale_score(0, wrapped_score),
            "at 0 moves, the scale is SCALE_MAX"
        );

        assert_eq!(
            0.0,
            scale_negative.scale_score(
                MovesScaler::MOVES_WHEN_WELL_INTO_GAME as usize,
                wrapped_score
            ),
            "at MOVES_WHEN_WELL_INTO_GAME moves, the scale is 0"
        );

        assert_eq!(
            wrapped_score,
            scale_negative.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME / 2.0) as usize,
                wrapped_score
            ),
            "at half of MOVES_WHEN_WELL_INTO_GAME moves, the scale is 1"
        );

        assert!(
            scale_negative.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.6) as usize,
                wrapped_score
            ) < wrapped_score - 2.0,
            "at 60% into the core game, the scale is nudging the score down"
        );

        assert!(
            scale_negative.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.25) as usize,
                wrapped_score
            ) > 12.0,
            "at 25% into the core game, the scale is still very high"
        );

        assert!(
            scale_negative.scale_score(
                (MovesScaler::MOVES_WHEN_WELL_INTO_GAME * 0.10) as usize,
                wrapped_score
            ) > MovesScaler::SCALE_MAX * wrapped_score - 1.0,
            "at 10% into the core game, the scale is effectively SCALE_MAX"
        );
    }
}
