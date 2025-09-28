use rand::rngs::ThreadRng;
use std::fmt;

use crate::draw_pile::DrawPile;

use crate::board_state::BoardState;
pub use crate::board_state::Direction;

use crate::board_state::Card;

#[derive(PartialEq)]
pub struct GameState {
    board: BoardState,
    draw_pile: DrawPile,
    next: Card,
}

impl GameState {
    pub fn initialize(rng: &mut ThreadRng) -> Self {
        let mut draw_pile = DrawPile::initialize(rng);

        let board = BoardState::initialize(&mut draw_pile, rng);

        let next = draw_pile.draw(rng);

        GameState {
            board,
            draw_pile,
            next,
        }
    }

    pub fn shift(&mut self, dir: Direction, rng: &mut ThreadRng) -> Option<Self> {
        let new_board = self.board.shift(dir, self.next, rng)?;
        let mut new_draw_pile = self.draw_pile.clone();
        let new_next = new_draw_pile.draw(rng);
        Some(GameState {
            board: new_board,
            draw_pile: new_draw_pile,
            next: new_next,
        })
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Next: {}\n{}", self.next, self.board)
    }
}
impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.board, self.draw_pile)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn initialize() {
        let mut rng = thread_rng();
        let game_state = GameState::initialize(&mut rng);
        crate::board_state::tests::assert_board_values(&game_state.board);

        assert!(game_state.next > 0, "'next' has a card");
        assert_eq!(
            2,
            game_state.draw_pile.len(),
            "the draw pile has been drawn down to 2 cards"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn shift() {
        let mut rng = thread_rng();

        let mut grid = [0; 16];
        grid[1] = 1;
        let board = BoardState::initialize_test_state(grid);

        let mut draw_pile = DrawPile::initialize_test_pile(vec![9, 6, 3]);
        let next = draw_pile.draw(&mut rng);

        let mut game_state = GameState {
            board,
            draw_pile,
            next,
        };

        let mut new_state = game_state.shift(Direction::Left, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            1, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(expected, new_state.board, "board shifted left as expected");
        assert_eq!(6, new_state.next, "next card was drawn");
        assert_eq!(DrawPile::initialize_test_pile(vec![9]), new_state.draw_pile, "card was drawn from the draw pile");

        let no_state = new_state.shift(Direction::Up, &mut rng);
        assert_eq!(None, no_state, "board did not shift");
        assert_eq!(expected, new_state.board, "prior board did not shift");
        assert_eq!(6, new_state.next, "prior board did not draw a new card");
    }
}
