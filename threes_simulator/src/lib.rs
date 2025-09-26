use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

type Card = u16;

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
