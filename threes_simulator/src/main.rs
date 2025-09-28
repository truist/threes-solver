use rand::thread_rng;

use threes_simulator::{Direction, GameState};

fn main() {
    let mut rng = thread_rng();
    let mut game = GameState::initialize(&mut rng);
    println!("{game}");
    let game = game.shift(Direction::Left, &mut rng).unwrap();
    println!("{game}");
}
