use std::fmt;

use rng_util::{IteratorRandom, RngType};

use crate::draw_pile::DrawPile;
pub use crate::draw_pile::DrawType;

use crate::board_state::BoardState;
pub use crate::board_state::{Card, Direction, Grid};

#[derive(Clone, PartialEq)]
pub struct GameState {
    board: BoardState,
    draw_pile: DrawPile,
    next: DrawType,
    moves: usize,
}

impl GameState {
    pub fn initialize(rng: &mut RngType) -> Self {
        let mut draw_pile = DrawPile::initialize(rng);

        let board = BoardState::initialize(&mut draw_pile, rng);

        let next = draw_pile.draw(rng);

        Self {
            board,
            draw_pile,
            next,
            moves: 0,
        }
    }

    #[cfg(any(test, feature = "workspace_test"))]
    pub fn initialize_test_state(board: BoardState, draw_pile: DrawPile, next: DrawType) -> Self {
        Self {
            board,
            draw_pile,
            next,
            moves: 0,
        }
    }

    pub fn shift(&self, dir: Direction, choose_next: bool, rng: &mut RngType) -> Option<Self> {
        let next = if choose_next {
            Some(self.choose_next_card(rng))
        } else {
            None
        };

        // note the trailing ?
        let board = self.board.shift(dir, next, rng)?;
        Some(self.shift_new(board, rng))
    }

    pub fn shift_all(&self, dir: Direction, rng: &mut RngType) -> Vec<Self> {
        let next_cards = match self.next {
            DrawType::Regular(card) => vec![card],
            DrawType::Bonus(cards) => cards.to_vec(),
        };
        self.board
            .shift_all(dir, next_cards)
            .into_iter()
            .map(|board| self.shift_new(board, rng))
            .collect()
    }

    fn choose_next_card(&self, rng: &mut RngType) -> Card {
        match self.next {
            DrawType::Regular(card) => card,
            DrawType::Bonus(cards) => *cards.iter().choose(rng).unwrap(),
        }
    }

    fn shift_new(&self, board: BoardState, rng: &mut RngType) -> Self {
        let mut draw_pile = self.draw_pile.clone();
        draw_pile.new_high_card(board.high_card());

        let next = draw_pile.draw(rng);

        Self {
            board,
            draw_pile,
            next,
            moves: self.moves + 1,
        }
    }

    pub fn get_grid(&self) -> &Grid {
        self.board.get_grid()
    }

    pub fn get_next(&self) -> &DrawType {
        &self.next
    }

    pub fn high_card(&self) -> &Card {
        &self.board.high_card()
    }

    pub fn get_moves(&self) -> usize {
        self.moves
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
        write!(
            f,
            "Next: {}\n{}\nMoves: {}",
            next_colorized, self.board, self.moves
        )
    }
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}\n{}", self.board, self.draw_pile, self.moves)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    use rng_util::test_rng;

    #[test]
    fn initialize() {
        let mut rng = test_rng();
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
        let mut rng = test_rng();

        let mut grid = [0; 16];
        grid[1] = 1;
        let board = BoardState::initialize_test_state(grid, 1);

        let mut draw_pile = DrawPile::initialize_test_pile(vec![24, 12, 6, 3]);
        let next = draw_pile.draw(&mut rng);

        let game_state = GameState {
            board,
            draw_pile,
            next,
            moves: 0,
        };

        let new_state = game_state.shift(Direction::Left, true, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            1, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 1);
        assert_eq!(expected, new_state.board, "board shifted left as expected");
        assert_eq!(6, new_state.next.unwrap_regular(), "next card was drawn");
        assert_eq!(DrawPile::initialize_test_pile(vec![24, 12]), new_state.draw_pile, "card was drawn from the draw pile");
        assert_eq!(1, new_state.get_moves(), "1 move was logged");

        let no_state = new_state.shift(Direction::Up, false, &mut rng);
        assert_eq!(None, no_state, "board did not shift");
        assert_eq!(expected, new_state.board, "prior board did not shift");
        assert_eq!(6, new_state.next.unwrap_regular(), "prior board did not draw a new card");
        assert_eq!(1, new_state.get_moves(), "prior board did not log a move");

        let second_state = new_state.shift(Direction::Left, true, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            1, 0, 3, 6,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], 1);
        assert_eq!(expected, second_state.board, "board shifted left as expected");
        assert_eq!(12, second_state.next.unwrap_regular(), "next card was drawn");
        assert_eq!(DrawPile::initialize_test_pile(vec![24]), second_state.draw_pile, "card was drawn from the draw pile");
        assert_eq!(2, second_state.get_moves(), "another move was logged");
    }

    #[test]
    #[rustfmt::skip]
    fn shift_with_bonus() {
        let mut rng = test_rng();

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
            moves: 0,
        };

        let new_state = game_state.shift(Direction::Left, true, &mut rng).unwrap();
        let expected = BoardState::initialize_test_state([
            192, 0, 0, game_state.next.unwrap_regular(),
              0, 0, 0, 0,
              0, 0, 0, 0,
              0, 0, 0, 0,
        ], 192);
        assert_eq!(expected, new_state.board, "board shifted left as expected");
        assert_eq!(3, new_state.draw_pile.len().1, "bonus pile has been populated");
    }

    #[test]
    #[rustfmt::skip]
    fn shift_all() {
        let mut rng = test_rng();

        let board = BoardState::initialize_test_state([
            0,  3, 0, 0,
            0,  6, 0, 0,
            0, 12, 0, 0,
            0, 24, 0, 0,
        ], 1);

        let draw_pile = DrawPile::initialize_test_pile(vec![24, 12, 6, 3]);
        let next = DrawType::Bonus([4, 7, 13]);

        let game_state = GameState {
            board,
            draw_pile,
            next,
            moves: 0,
        };

        let new_states = game_state.shift_all(Direction::Left, &mut rng);
        let mut ns_iter = new_states.iter();
        assert_eq!(12, new_states.len(), "Got all 12 possible shifts");
        for row in 0..4 {
            for card in [4, 7, 13] {
                let mut inserted_row_values = [0; 4];
                inserted_row_values[row] = card;
                let expected = BoardState::initialize_test_state([
                     3, 0, 0, inserted_row_values[0],
                     6, 0, 0, inserted_row_values[1],
                    12, 0, 0, inserted_row_values[2],
                    24, 0, 0, inserted_row_values[3],
                ], 1);
                let next_state = ns_iter.next().unwrap();
                assert_eq!(expected, next_state.board, "board row {row} card {card} shifted left as expected");
                assert_eq!(3, next_state.next.unwrap_regular(), "next card was drawn for board row {row} card {card}");
                assert_eq!(DrawPile::initialize_test_pile(vec![24, 12, 6]), next_state.draw_pile, "card was drawn from the draw pile for board row {row} card {card}");
                assert_eq!(1, next_state.get_moves(), "1 move was logged for board row {row} card {card}");
            }
        }

        let no_new_states = new_states[0].shift_all(Direction::Up, &mut rng);
        assert_eq!(0, no_new_states.len(), "no new states after shifting up");
    }
}
