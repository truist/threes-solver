use threes_simulator::game_state::GameState;

use crate::algo::core::Algo;

use super::super::core::ValueBooster;

// cells that are empty
#[derive(Debug)]
pub(crate) struct Empties;

impl Algo for Empties {
    fn score(&self, game_state: &GameState, booster: Option<&dyn ValueBooster>) -> f64 {
        self.assert_value_booster_not_supported(booster);

        game_state
            .get_grid()
            .iter()
            .map(|&card| if card > 0 { 0.0 } else { 1.0 })
            .sum::<f64>()
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
    fn test_score() {
        let mut grid = [0; 16];
        grid[1] = 1;
        let game_state = generate_game_state(grid);

        assert!(
            Empties.score(&game_state, None) > 0.0,
            "with a valid GameState, the score is greater than 0"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_empties() {
        let game_state = generate_game_state([
            3, 3, 3, 0,
            0, 3, 3, 3,
            0, 3, 3, 0,
            0, 0, 0, 0,
        ]);

        assert_eq!(8.0, Empties.score(&game_state, None), "empty cells are counted correctly");
    }

    #[test]
    #[rustfmt::skip]
    fn test_max_possible_score() {
        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(16.0, Empties.score(&game_state, None), "empties max score");
    }
}
