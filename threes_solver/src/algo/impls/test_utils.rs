#![cfg(test)]

use rng_util::test_rng;
use threes_simulator::board_state::BoardState;
use threes_simulator::draw_pile::DrawPile;
use threes_simulator::game_state::GameState;
use threes_simulator::game_state::Grid;

pub(crate) fn generate_game_state(grid: Grid) -> GameState {
    let mut rng = test_rng();
    let mut draw_pile = DrawPile::initialize(&mut rng);
    let next = draw_pile.draw(&mut rng);

    let board_state = BoardState::initialize_test_state(grid, *grid.iter().max().unwrap());

    GameState::initialize_test_state(board_state, draw_pile, next)
}
