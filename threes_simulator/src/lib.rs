pub mod board_state;
mod draw_pile;
pub mod game_state;

use crossterm::event::{read, Event, KeyCode};
use rand::thread_rng;

use crate::game_state::{Direction, GameState};

pub fn play() {
    println!("Press q to quit");
    println!("Use arrow keys to shift the board");

    let mut rng = thread_rng();
    let mut game = GameState::initialize(&mut rng);

    println!("{game}");
    loop {
        println!("");
        crossterm::terminal::enable_raw_mode().unwrap();
        if let Event::Key(event) = read().unwrap() {
            crossterm::terminal::disable_raw_mode().unwrap();
            if let Some(dir) = match event.code {
                KeyCode::Up => Some(Direction::Up),
                KeyCode::Right => Some(Direction::Right),
                KeyCode::Down => Some(Direction::Down),
                KeyCode::Left => Some(Direction::Left),
                KeyCode::Char('q') => break, // quit on 'q'
                _ => None,
            } {
                println!("You pressed {dir}");
                if let Some(new_game) = game.shift(dir, &mut rng) {
                    game = new_game;
                    println!("{game}");
                } else {
                    println!("Impossible");
                }
            } else {
                println!("Key not understood. (Press q to quit)");
            }
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}
