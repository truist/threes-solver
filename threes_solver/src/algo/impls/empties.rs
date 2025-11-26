use threes_simulator::game_state::GameState;

// cells that are empty
pub(crate) fn empties(game_state: &GameState) -> u8 {
    game_state
        .get_grid()
        .iter()
        .map(|&card| if card > 0 { 0 } else { 1 })
        .sum::<u8>()
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos;
    use crate::Algo;

    use super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    fn test_score() {
        assert_eq!(
            0,
            Algos::Empties.score(&None),
            "all 'None' states get a 0 score"
        );

        let mut grid = [0; 16];
        grid[1] = 1;
        let game_state = generate_game_state(grid);

        assert!(
            Algos::Empties.score(&Some(game_state)) > 0,
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

        assert_eq!(8, empties(&game_state), "empty cells are counted correctly");
    }
}
