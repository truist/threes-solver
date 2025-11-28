use threes_simulator::game_state::GameState;

use crate::algo::core::{AlgoScalers, Algos};

impl Algos {
    // cells that are empty
    pub(crate) fn empties(&self, game_state: &GameState, scalers: &AlgoScalers) -> f64 {
        game_state
            .get_grid()
            .iter()
            .map(|&card| {
                if card > 0 {
                    0.0
                } else {
                    scalers.scale_score(1.0, game_state, &[card])
                }
            })
            .sum::<f64>()
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::AlgoScalers;
    use crate::algo::core::Algos::Empties;
    use crate::algo::wrappers::ValueScaler;
    use crate::Algo;

    use super::super::super::test_utils::generate_game_state;

    #[test]
    fn test_score() {
        let no_scalers = &AlgoScalers { scalers: vec![] };

        assert_eq!(
            0.0,
            Empties.score(&None, no_scalers),
            "all 'None' states get a 0 score"
        );

        let mut grid = [0; 16];
        grid[1] = 1;
        let game_state = generate_game_state(grid);

        assert!(
            Empties.score(&Some(game_state), no_scalers) > 0.0,
            "with a valid GameState, the score is greater than 0"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_empties() {
        let no_scalers = &AlgoScalers { scalers: vec![] };
        let game_state = generate_game_state([
            3, 3, 3, 0,
            0, 3, 3, 3,
            0, 3, 3, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(8.0, Empties.empties(&game_state, no_scalers), "empty cells are counted correctly");

        let test_scale = 3.0;
        let scale_zeros = ValueScaler {
            min_value_to_scale: 0,
            max_value_to_scale: 0,
            scale: test_scale,
        };
        let scalers = &AlgoScalers { scalers: vec![&scale_zeros] };
        assert_eq!(
            8.0 * test_scale, Empties.empties(&game_state, scalers),
            "scaling works, even in this contrived example"
        );
    }
}
