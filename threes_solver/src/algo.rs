use threes_simulator::game_state::{Direction, GameState};

pub enum Algos {
    Empties,
}

impl Algos {
    pub fn score(&self, game_state: &Option<GameState>, _last_move_dir: &Direction) -> f64 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state),
            }
        } else {
            0.0
        }
    }

    // TODO: should this return 0 when the grid is full?
    // or some positive integer (e.g. 1) because even when full there still might be a move?
    // a 0 value will zero out a weight multiplier...
    fn empties(&self, game_state: &GameState) -> f64 {
        game_state
            .get_grid()
            .iter()
            .map(|&card| if card > 0 { 0 } else { 1 })
            .sum::<u8>() as f64
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;
    use threes_simulator::game_state::Grid;

    #[test]
    fn test_score() {
        assert_eq!(
            0.0,
            Algos::Empties.score(&None, &Direction::Left),
            "all 'None' states get a 0 score"
        );

        let mut grid = [0; 16];
        grid[1] = 1;
        let game_state = generate_game_state(grid);

        assert!(
            Algos::Empties.score(&Some(game_state), &Direction::Left) > 0.0,
            "with a valid GameState, the score is greater than 0"
        );
    }

    fn generate_game_state(grid: Grid) -> GameState {
        let mut rng = thread_rng();
        let mut draw_pile = DrawPile::initialize(&mut rng);
        let next = draw_pile.draw(&mut rng);

        let board_state = BoardState::initialize_test_state(grid, *grid.iter().max().unwrap());

        GameState::initialize_test_state(board_state, draw_pile, next)
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

        assert_eq!(8.0, Algos::Empties.empties(&game_state), "empty cells are counted correctly");
    }
}
