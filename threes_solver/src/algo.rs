use threes_simulator::game_state::{Card, Direction, GameState};

pub enum Algos {
    Empties,
}

impl Algos {
    pub fn score(
        &self,
        game_state: &Option<GameState>,
        _last_move_dir: &Direction,
        _next: &Card,
    ) -> f64 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state),
            }
        } else {
            0.0
        }
    }

    fn empties(&self, game_state: &GameState) -> f64 {
        game_state
            .get_grid()
            .iter()
            .map(|&card| if card > 0 { 0 } else { 1 })
            .sum::<u8>() as f64
    }
}
