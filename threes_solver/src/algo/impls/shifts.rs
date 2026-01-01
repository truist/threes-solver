use strum::IntoEnumIterator;
use threes_simulator::game_state::{Direction, GameState};

use crate::algo::core::Algo;

use super::super::core::ValueBooster;

// counts how many rows/cols would shift across all directions
#[derive(Debug)]
pub(crate) struct Shifts;

impl Algo for Shifts {
    fn score(&self, game_state: &GameState, booster: Option<&dyn ValueBooster>) -> f64 {
        self.assert_value_booster_not_supported(booster);

        Direction::iter()
            .map(|dir| game_state.shift_mask(dir).count_ones() as f64)
            .sum()
    }

    fn normalization_factor(&self) -> f64 {
        super::ALGO_MAX_BASE / 16.0
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::super::test_utils::generate_game_state;

    #[test]
    #[rustfmt::skip]
    fn test_score_counts_shifted_rows_cols() {
        let game_state = generate_game_state([
            0, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);

        assert_eq!(3.0, Shifts.score(&game_state, None), "one row shifts left/right and one column shifts down");
    }

    #[test]
    #[rustfmt::skip]
    fn test_max_possible_score() {
        let game_state = generate_game_state([
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
        ]);

        assert_eq!(16.0, Shifts.score(&game_state, None), "shifts max score");

        assert_eq!(
            super::super::ALGO_MAX_BASE,
            Shifts.score(&game_state, None) * Shifts.normalization_factor(),
            "normalization of the highest score gives the overall max score"
        );
    }
}
