use rand::thread_rng;
use std::fmt;

mod draw_pile;
use draw_pile::DrawPile;

mod board_state;
use board_state::BoardState;

type Card = u16;

pub struct GameState {
    board: BoardState,
    draw_pile: DrawPile,
}

impl GameState {
    pub fn initialize() -> Self {
        let mut rng = thread_rng();

        let mut draw_pile = DrawPile::initialize(&mut rng);

        let board = BoardState::initialize(&mut draw_pile, &mut rng);

        GameState { board, draw_pile }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.board, self.draw_pile)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize() {
        let game_state = GameState::initialize();
        board_state::tests::assert_board_values(&game_state.board);
    }
}
