use std::fmt;

use threes_simulator::game_state::GameState;

use crate::algo::core::{Algo, ValueBooster};
use crate::algo::impls::empties::Empties;

#[derive(Debug)]
pub(crate) struct FewEmptiesScaler {
    pub(crate) wrapped: Box<dyn Algo>,
}

impl FewEmptiesScaler {
    const SCALE_MAX: f64 = 2.0;
    const EMPTIES_THRESHOLD: f64 = 5.0;

    fn scale_score(&self, empties: f64, base_score: f64) -> f64 {
        // 6 empties -> 0 factor -> 0% boost -> 100%
        // 5 empties -> 1 factor -> 17% boost -> 117%
        // 4 empties -> 2 factor -> 33% boost -> 133%
        // 3 empties -> 3 factor -> 50% boost -> 150%
        // 2 empties -> 4 factor -> 67% boost -> 167%
        // 1 empties -> 5 factor -> 83% boost -> 183%
        // 0 empties -> 6 factor -> 100% boost -> 200%
        let factor = (Self::EMPTIES_THRESHOLD + 1.0 - empties).max(0.0);
        let boost = factor / (Self::EMPTIES_THRESHOLD + 1.0) * (Self::SCALE_MAX - 1.0);
        let percentage = 1.0 + boost;
        base_score * percentage
    }
}

impl Algo for FewEmptiesScaler {
    fn score(&self, game_state: &GameState, value_booster: Option<&dyn ValueBooster>) -> f64 {
        let base_score = self.wrapped.score(game_state, value_booster);
        let empties = Empties.score(game_state, None);

        self.scale_score(empties, base_score)
    }

    fn normalization_factor(&self) -> f64 {
        // see comment in ValueBooster.normalization_factor()
        self.wrapped.normalization_factor() / Self::SCALE_MAX
    }
}

impl fmt::Display for FewEmptiesScaler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (scaled w/empties)", self.wrapped)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_empties_scale_score() {
        let empties_scaler = FewEmptiesScaler {
            wrapped: Box::new(Empties), // doesn't matter what this is
        };

        let wrapped_score = 7.0;

        assert_eq!(
            wrapped_score,
            empties_scaler.scale_score(10.0, wrapped_score),
            "with many empties, the score is unmodified"
        );

        assert_eq!(
            wrapped_score * FewEmptiesScaler::SCALE_MAX,
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
        assert_eq!(3.0, Empties.score(&game_state, None), "empties() does what we think it does")
    }
}
