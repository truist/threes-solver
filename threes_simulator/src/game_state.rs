use rand::seq::IteratorRandom;
use rand::Rng;
use std::fmt;

use crate::draw_pile::DrawPile;
pub use crate::draw_pile::DrawType;

use crate::board_state::BoardState;
pub use crate::board_state::{Card, Direction, Grid};

#[derive(Clone, PartialEq)]
pub struct GameState {
    board: BoardState,
    draw_pile: DrawPile,
    next: DrawType,
}

impl GameState {
    pub fn initialize<R: Rng>(rng: &mut R) -> Self {
        let mut draw_pile = DrawPile::initialize(rng);

        let board = BoardState::initialize(&mut draw_pile, rng);

        let next = draw_pile.draw(rng);

        GameState {
            board,
            draw_pile,
            next,
        }
    }

    #[cfg(any(test, feature = "workspace_test"))]
    pub fn initialize_test_state(board: BoardState, draw_pile: DrawPile, next: DrawType) -> Self {
        GameState {
            board,
            draw_pile,
            next,
        }
    }

    pub fn shift<R: Rng>(&self, dir: Direction, rng: &mut R) -> Option<Self> {
        let next = match self.next {
            DrawType::Regular(card) => card,
            DrawType::Bonus(cards) => *cards.iter().choose(rng).unwrap(),
        };
        let mut new_board = self.board.shift(dir, next, rng)?;

        let mut new_draw_pile = self.draw_pile.clone();
        new_draw_pile.new_high_card(new_board.high_card());

        let new_next = new_draw_pile.draw(rng);

        Some(GameState {
            board: new_board,
            draw_pile: new_draw_pile,
            next: new_next,
        })
    }

    pub fn get_grid(&self) -> &Grid {
        self.board.get_grid()
    }

    pub fn get_next(&self) -> &DrawType {
        &self.next
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let next_colorized = match self.next {
            DrawType::Regular(card) => {
                let next_as_str = card.to_string();
                BoardState::colorize(card, &next_as_str).to_string()
            }
            DrawType::Bonus(cards) => cards
                .iter()
                .map(|&card| {
                    let next_as_str = card.to_string();
                    BoardState::colorize(card, &next_as_str).to_string()
                })
                .collect::<Vec<String>>()
                .join(", "),
        };
        write!(f, "Next: {}\n{}", next_colorized, self.board)
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

        assert!(game_state.next.unwrap_regular() > 0, "'next' has a card");
        assert_eq!(
            2,
            game_state.draw_pile.len().0,
            "the draw pile has been drawn down to 2 cards"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn shift() {
        let mut rng = thread_rng();

        let mut grid = [0; 16];
        grid[1] = 1;
        let board = BoardState::initialize_test_state(grid, 1);

        let mut draw_pile = DrawPile::initialize_test_pile(vec![9, 6, 3]);
        let next = draw_pile.draw(&mut rng);

        let game_state = GameState {
            board,
            draw_pile,
            next,
        };

        let new_state = game_state.shift(Direction::Left, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            1, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 1);
        assert_eq!(expected, new_state.board, "board shifted left as expected");
        assert_eq!(6, new_state.next.unwrap_regular(), "next card was drawn");
        assert_eq!(DrawPile::initialize_test_pile(vec![9]), new_state.draw_pile, "card was drawn from the draw pile");

        let no_state = new_state.shift(Direction::Up, &mut rng);
        assert_eq!(None, no_state, "board did not shift");
        assert_eq!(expected, new_state.board, "prior board did not shift");
        assert_eq!(6, new_state.next.unwrap_regular(), "prior board did not draw a new card");
    }

    #[test]
    #[rustfmt::skip]
    fn shift_with_bonus() {
        let mut rng = thread_rng();

        let mut grid = [0; 16];
        grid[0] = 96;
        grid[1] = 96;
        let board = BoardState::initialize_test_state(grid, 96);

        let mut draw_pile = DrawPile::initialize(&mut rng);
        assert_eq!(0, draw_pile.len().1);
        let next = draw_pile.draw(&mut rng);

        let game_state = GameState {
            board,
            draw_pile,
            next,
        };

        let new_state = game_state.shift(Direction::Left, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            192, 0, 0, game_state.next.unwrap_regular(),
              0, 0, 0, 0,
              0, 0, 0, 0,
              0, 0, 0, 0,
        ], 192);
        assert_eq!(expected, new_state.board, "board shifted left as expected");
        assert_eq!(3, new_state.draw_pile.len().1, "bonus pile has been populated");
    }
}
