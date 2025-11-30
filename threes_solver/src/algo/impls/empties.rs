use threes_simulator::game_state::GameState;

use crate::algo::core::Algos;

use super::super::core::{assert_value_booster_not_supported, ValueBooster};

impl Algos {
    // cells that are empty
    pub(crate) fn empties(
        &self,
        game_state: &GameState,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
        assert_value_booster_not_supported(self, booster);

        game_state
            .get_grid()
            .iter()
            .map(|&card| if card > 0 { 0.0 } else { 1.0 })
            .sum::<f64>()
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos::Empties;
    use crate::Algo;

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

        assert_eq!(8.0, Empties.empties(&game_state, None), "empty cells are counted correctly");
    }
}
