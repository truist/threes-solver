use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

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
    fn score(&self, game_state: &Option<GameState>) -> i8 {
        if let Some(actual_game_state) = game_state {
            let base_score = self.wrapped.score(game_state);
            self.scale_score(actual_game_state.get_moves(), base_score)
        } else {
            0
        }
    }
}
impl<A: Algo> fmt::Display for MovesScaled<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = if self.positive { "📈" } else { "📉" };
        write!(f, "{:?} ({})", self.wrapped, dir)
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
}
