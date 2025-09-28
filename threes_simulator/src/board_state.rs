use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fmt;

use crate::draw_pile::DrawPile;
use crate::Card;

const BOARD_SIZE: usize = 4;

type Grid = [Card; 16];

#[cfg(test)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable = match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };
        write!(f, "{readable}")
    }
}

#[derive(PartialEq)]
pub struct BoardState {
    grid: Grid,
}

impl BoardState {
    pub fn initialize(draw_pile: &mut DrawPile, rng: &mut ThreadRng) -> Self {
        let mut grid: Vec<Card> = (0..9).map(|_| draw_pile.draw(rng)).collect();
        let mut empties = vec![0; 7];
        grid.append(&mut empties);
        grid.shuffle(rng);

        let grid: Grid = grid.try_into().unwrap();
        BoardState { grid }
    }

    #[cfg(test)]
    pub fn shift(&self, dir: &Direction) -> Option<BoardState> {
        let (outer_start, outer_incr, inner_start, inner_incr): (isize, isize, isize, isize) =
            match dir {
                Direction::Left => (0, 4, 0, 1),
                Direction::Up => (3, -1, 0, 4),
                Direction::Right => (12, -4, 3, -1),
                Direction::Down => (0, 1, 12, -4),
            };

        let mut new_grid = self.grid.clone();
        let mut shift_happened = false;

        for outer_round in 0..BOARD_SIZE {
            let outer = outer_start + outer_incr * outer_round as isize;

            for inner_round in 0..BOARD_SIZE - 1 {
                let inner = inner_start + inner_incr * inner_round as isize;

                let cur = outer + inner;
                let cur = cur as usize;

                let next = outer + inner + inner_incr;
                let next = next as usize;

                if new_grid[cur] == 0 {
                    new_grid[cur] = new_grid[next];
                    new_grid[next] = 0;
                    shift_happened = true;
                } else if new_grid[cur] >= 3 {
                    if new_grid[cur] == new_grid[next] {
                        new_grid[cur] *= 2;
                        new_grid[next] = 0;
                        shift_happened = true;
                    }
                } else {
                    // 1 or 2
                    if new_grid[cur] + new_grid[next] == 3 {
                        new_grid[cur] = 3;
                        new_grid[next] = 0;
                        shift_happened = true;
                    }
                }
            }
        }

        shift_happened.then_some(BoardState { grid: new_grid })
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                let cell = self.grid[r * BOARD_SIZE + c];
                let value = match cell {
                    0 => ".".to_string(),
                    _ => cell.to_string(),
                };
                write!(f, "{value:^4}")?;
            }
            if r < 3 {
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "")?;
        BoardState::fmt(&self, f)
    }
}
impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BoardState::fmt(&self, f)
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

        assert_ne!(board1.grid, board2.grid, "boards are shuffled");

        assert_board_values(&board1);
    }

    pub fn assert_board_values(board: &BoardState) {
        let (mut zeros, mut ones, mut twos, mut threes) = (0, 0, 0, 0);
        for &card in board.grid.iter() {
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
    #[rustfmt::skip]
    fn shift() {
        let before = [
            0, 3, 0, 3,
            0, 0, 3, 3,
            3, 0, 3, 0,
            3, 6, 0, 3,
        ];
        let after = [
            3, 0, 3, 0,
            0, 3, 3, 0,
            3, 3, 0, 0,
            3, 6, 3, 0,
        ];
        test_shift(before, after, "all the basic shift cases, no merges");

        let before = [
            3, 3, 0, 0,
            3, 3, 3, 3,
            3, 6, 12, 24,
            0, 0, 0, 0,
        ];
        let after = [
            6, 0, 0, 0,
            6, 3, 3, 0,
            3, 6, 12, 24,
            0, 0, 0, 0,
        ];
        test_shift(before, after, "all the basic merge cases");

        let before = [
            1, 1, 0, 0,
            2, 2, 0, 0,
            1, 2, 0, 0,
            2, 1, 0, 0,
        ];
        let after = [
            1, 1, 0, 0,
            2, 2, 0, 0,
            3, 0, 0, 0,
            3, 0, 0, 0,
        ];
        test_shift(before, after, "1 and 2 are special");

        let before = [
            1, 3, 1, 0,
            2, 3, 2, 0,
            1, 2, 3, 0,
            1, 2, 1, 2,
        ];
        let after = [
            1, 3, 1, 0,
            2, 3, 2, 0,
            3, 3, 0, 0,
            3, 1, 2, 0,
        ];
        test_shift(before, after, "1 and 2 and 3");

        let before = [
             3,  6, 12, 24,
            24, 12,  6,  3,
             3,  6, 12, 24,
             3,  6,  3,  6,
        ];
        let start_state = BoardState { grid: before };
        assert_eq!(None, start_state.shift(&Direction::Left), "get a None when nothing can move: left");
        let before = rotate_right(&before);
        let start_state = BoardState { grid: before };
        assert_eq!(None, start_state.shift(&Direction::Up), "get a None when nothing can move: up");
    }

    fn test_shift(before: Grid, after: Grid, desc: &str) {
        test_shift_direction(Direction::Left, before, after, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Up, before, after, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Right, before, after, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Down, before, after, desc);
    }

    fn test_shift_direction(dir: Direction, before: Grid, after: Grid, desc: &str) {
        let start_state = BoardState { grid: before };
        let end_state = start_state.shift(&dir).unwrap();
        let expected = BoardState { grid: after };
        assert_eq!(
            expected, end_state,
            "{desc}: {dir}, from start state:\n{start_state}"
        );
    }

    fn rotate_right(orig: &Grid) -> Grid {
        let mut rotated = [0; 16];
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                rotated[c * BOARD_SIZE + (3 - r)] = orig[r * BOARD_SIZE + c];
            }
        }
        rotated
    }

    #[test]
    #[rustfmt::skip]
    fn test_rotate_right() {
        let start = [
             0,  1,  2,  3,
             4,  5,  6,  7,
             8,  9, 10, 11,
            12, 13, 14, 15,
        ];
        let expected = [
            12,  8, 4, 0,
            13,  9, 5, 1,
            14, 10, 6, 2,
            15, 11, 7, 3,
        ];
        assert_eq!(expected, rotate_right(&start), "rotate_right is correct");
    }
}
