use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

type Card = u16;

#[derive(Debug)]
struct DrawPile {
    cards: Vec<Card>,
}

impl DrawPile {
    fn initialize(rng: &mut ThreadRng, main_pile: bool) -> DrawPile {
        match main_pile {
            false => DrawPile { cards: vec![] },
            true => {
                let mut cards = vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
                cards.shuffle(rng);
                DrawPile { cards }
            }
        }
    }
}

impl Iterator for DrawPile {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        self.cards.pop()
    }
}

impl fmt::Display for DrawPile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.cards)
    }
}

// #[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[derive(Debug)]
struct BoardState {
    board: [[Card; 4]; 4],
}

impl BoardState {
    fn initialize(draw_pile: &mut DrawPile, rng: &mut ThreadRng) -> BoardState {
        let mut board: Vec<Card> = draw_pile.take(9).collect();
        let mut empties = vec![0; 7];
        board.append(&mut empties);
        board.shuffle(rng);

        let board: [[Card; 4]; 4] = [
            board[0..4].try_into().unwrap(),
            board[4..8].try_into().unwrap(),
            board[8..12].try_into().unwrap(),
            board[12..16].try_into().unwrap(),
        ];

        BoardState { board }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (r, row) in self.board.iter().enumerate() {
            let mut value;
            for cell in row {
                if *cell == 0 {
                    value = ".".to_string();
                } else {
                    value = cell.to_string();
                }
                write!(f, "{value:^4}")?;
            }
            if r < 3 {
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GameState {
    rng: ThreadRng,
    board: BoardState,
    draw_pile: DrawPile,
    bonus_pile: DrawPile,
}

impl GameState {
    pub fn initialize() -> GameState {
        let mut rng = thread_rng();

        let mut draw_pile = DrawPile::initialize(&mut rng, true);

        let board = BoardState::initialize(&mut draw_pile, &mut rng);

        let bonus_pile = DrawPile::initialize(&mut rng, false);

        GameState {
            rng,
            board,
            draw_pile,
            bonus_pile,
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}\n{}", self.board, self.draw_pile, self.bonus_pile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_pile_initialize() {
        let mut rng = thread_rng();

        let bonus_pile = DrawPile::initialize(&mut rng, false);
        assert_eq!(bonus_pile.cards.len(), 0, "bonus pile is empty");

        let main1 = DrawPile::initialize(&mut rng, true);
        assert_eq!(main1.cards.len(), 12, "main pile has 12 cards");

        for value in 1..=3 {
            let value_count = main1.cards.iter().filter(|each| **each == value).count();
            assert_eq!(4, value_count, "4 cards with value {value}");
        }

        let main2 = DrawPile::initialize(&mut rng, true);
        assert_ne!(main1.cards, main2.cards, "main piles are shuffled");
    }

    #[test]
    fn board_state_initialize() {
        let mut rng = thread_rng();

        let mut main1 = DrawPile::initialize(&mut rng, true);
        let board1 = BoardState::initialize(&mut main1, &mut rng);

        assert_eq!(3, main1.cards.len(), "draw pile had 9 cards removed");

        let mut main2 = DrawPile::initialize(&mut rng, true);
        let board2 = BoardState::initialize(&mut main2, &mut rng);

        assert_ne!(board1.board, board2.board, "boards are shuffled");

        assert_board_values(&board1);
    }

    fn assert_board_values(board: &BoardState) {
        let (mut zeros, mut ones, mut twos, mut threes) = (0, 0, 0, 0);
        for &card in board.board.iter().flatten() {
            match card {
                0 => zeros += 1,
                1 => ones += 1,
                2 => twos += 1,
                3 => threes += 1,
                _ => {
                    assert!(false, "this shouldn't be possible")
                }
            }
        }

        assert_eq!(7, zeros, "7 empty cards");
        assert!(ones > 0, "at least one 1");
        assert!(twos > 0, "at least one 2");
        assert!(threes > 0, "at least one 3");
        assert_eq!(9, ones + twos + threes, "9 non-empty cards");
    }

    #[test]
    #[should_panic]
    fn board_state_initialize_panic_on_short_pile() {
        let mut rng = thread_rng();
        let mut empty_pile = DrawPile::initialize(&mut rng, false);
        let _ = BoardState::initialize(&mut empty_pile, &mut rng);
    }

    #[test]
    fn game_state_initialize() {
        let game_state = GameState::initialize();
        assert_board_values(&game_state.board);
    }
}
