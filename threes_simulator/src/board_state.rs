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
                } else if new_grid[cur] == new_grid[next] {
                    new_grid[cur] *= 2;
                    new_grid[next] = 0;
                }
            }
        }

        Some(BoardState { grid: new_grid })
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

    fn test_shift(before: Grid, after: Grid, desc: &str) {
        test_shift_direction(Direction::Left, before, after, desc);

        let before = rotate_right(&before);
        let after = rotate_right(&after);
        test_shift_direction(Direction::Up, before, after, desc);

        let before = rotate_right(&before);
        let after = rotate_right(&after);
        test_shift_direction(Direction::Right, before, after, desc);

        let before = rotate_right(&before);
        let after = rotate_right(&after);
        test_shift_direction(Direction::Down, before, after, desc);
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

    fn test_shift_direction(dir: Direction, before: Grid, after: Grid, desc: &str) {
        let start_state = BoardState { grid: before };
        let end_state = start_state.shift(&dir).unwrap();
        let expected = BoardState { grid: after };
        assert_eq!(expected, end_state, "{desc}: {dir}");
    }

    #[test]
    #[rustfmt::skip]
    fn shift() {
        // before and after expected state
        // all four directions
        //  how? auto-rotate the cases?
        // cases:
        //  single card
        //  card at edge
        //  cards with a gap between
        //  cards that could merge but don't because there's space ahead of them
        //  cards that couldn't merge on an edge, while other cards move
        //  multiple merges
        //  just one merge per row/col
        //  whole shift can't be done
        //  test 1 + 2, in various cases
        // don't worry about the next card yet

        let before = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 0
        ];
        let after = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0
        ];
        test_shift(before, after, "the most basic shift of a single card");
    }
}
