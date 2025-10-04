/*
    TODO: more algos

    basics:
        mergable cards next to each other
            ditto, but for pairs that are "reachable", with adjustment for distance

        off-by-one cards next to each other
            with adjustment for distance? (requires "reachability")

        penalty for "trapped" numbers
            lower between two higher
            lower between a higher and a wall, with shifting blocked
            for having "too many" non-mergable cards next to you?
                especially for 1's and 2's?

    sort out how to handle variability of the next card (in solver.rs):
        options for where it might appear
            reward "risky" moves somehow? (e.g. to rescue trapped low-value cards, on the edge)
                this should fall out of how we handle the variability of the next card
        handling all the bonus options
        move-after-next when you can't know what card will appear
            maybe enumerate all the possibilities
            maybe do scoring where the next is unknown
            (maybe both, depending on lookahead depth / performance)
            any of these would be hard for a human to do

    cross-cutting:
        boost scores (and penalties) when it's 1's and 2's vs. other values
        or boost scores (and penalties) when it's high values?
        or both, and leave "mid" alone?

    advanced:
        just one card of the biggest size
            just one card of each of the biggest sizes

        bigger groups of empty spaces
        more "areas" of empty spaces

        high(er) values on a wall (vs. in the middle)
        high values in a corner (which is really just 2 walls)
        higher values clustered together
            maybe with extra bonus for being on a wall

        adjacent sequences of the same number
            (or penalize for separation... that seems equivalent)
            with some notion of "distance" and "reachability"?
        adjacent sequences of single increments (including 2 -> 1 -> 3)
            (ditto prior idea)
        bonus for multi-direction adjacency (gives more move options)
            maybe this comes out automatically with look-ahead?

    scores that need context beyond the current board state (in solver.rs):
        most moveable directions
        most future move possibilities (down a given path)
        best best-case future
        penalize having few future move possibilities
        penalize worst worst-case future
        "expect" a 1 or 2, "soon", based on:
            time since last 1/2 (both ways)
            1/2 imbalance (both ways)
*/

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

    fn generate_game_state(grid: Grid) -> GameState {
        let mut rng = thread_rng();
        let mut draw_pile = DrawPile::initialize(&mut rng);
        let next = draw_pile.draw(&mut rng);

        let board_state = BoardState::initialize_test_state(grid, *grid.iter().max().unwrap());

        GameState::initialize_test_state(board_state, draw_pile, next)
    }

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

        //TODO need a test for multiple algos
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
