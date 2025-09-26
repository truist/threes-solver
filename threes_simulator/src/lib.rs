use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

type Card = u16;

// #[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[derive(Debug)]
struct BoardState {
    board: [[Card; 4]; 4],
}

#[derive(Debug)]
struct DrawPile {
    cards: Vec<Card>,
}

impl Iterator for DrawPile {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        self.cards.pop()
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

        let mut draw_pile = vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
        draw_pile.shuffle(&mut rng);
        let mut draw_pile = DrawPile { cards: draw_pile };

        let bonus_pile = DrawPile { cards: vec![] };

        let board = GameState::initialize_board(&mut draw_pile, &mut rng);

        GameState {
            rng,
            board,
            draw_pile,
            bonus_pile,
        }
    }

    fn initialize_board(draw_pile: &mut DrawPile, rng: &mut ThreadRng) -> BoardState {
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
