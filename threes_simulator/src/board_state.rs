use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fmt;

use crate::draw_pile::DrawPile;
use crate::Card;

pub struct BoardState {
    board: [[Card; 4]; 4],
}

impl BoardState {
    pub fn initialize(draw_pile: &mut DrawPile, rng: &mut ThreadRng) -> Self {
        let mut board: Vec<Card> = (0..9).map(|_| draw_pile.draw(rng)).collect();
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

/************ tests *************/

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn initialize() {
        let mut rng = thread_rng();

        let mut main1 = DrawPile::initialize(&mut rng);
        let board1 = BoardState::initialize(&mut main1, &mut rng);

        assert_eq!(3, main1.len(), "draw pile had 9 cards removed");

        let mut main2 = DrawPile::initialize(&mut rng);
        let board2 = BoardState::initialize(&mut main2, &mut rng);

        assert_ne!(board1.board, board2.board, "boards are shuffled");

        assert_board_values(&board1);
    }

    pub fn assert_board_values(board: &BoardState) {
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
}
