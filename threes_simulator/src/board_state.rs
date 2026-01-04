use crossterm::style::{StyledContent, Stylize};
use std::fmt;
use std::string::ToString;
use strum_macros::{Display, EnumIter};

use rng_util::{IndexedRandom, RngType, SliceRandom};

use crate::draw_pile::DrawPile;

pub type Card = u16;

pub const BOARD_SIZE: usize = 4;

pub type Grid = [Card; 16];

#[derive(Clone, Copy, Debug, Display, EnumIter, PartialEq)]
pub enum Direction {
    Left = 0,
    Up = 1,
    Right = 2,
    Down = 3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardState {
    grid: Grid,
    high_card: Card,
}

impl BoardState {
    pub fn initialize(draw_pile: &mut DrawPile, rng: &mut RngType) -> Self {
        let mut grid: Vec<Card> = (0..9)
            .map(|_| draw_pile.draw(rng).unwrap_regular())
            .collect();
        let mut empties = vec![0; 7];
        grid.append(&mut empties);
        grid.shuffle(rng);

        let grid: Grid = grid.try_into().unwrap();
        Self { grid, high_card: 3 }
    }

    #[cfg(any(test, feature = "workspace_test"))]
    pub fn initialize_test_state(grid: Grid, high_card: Card) -> Self {
        Self { grid, high_card }
    }

    // Offsets and increments for the loops in shift(), depending on the desired direction
    fn offsets_for(&self, dir: Direction) -> (isize, isize, isize, isize) {
        match dir {
            Direction::Left => (0, 4, 0, 1),
            Direction::Up => (3, -1, 0, 4),
            Direction::Right => (12, -4, 3, -1),
            Direction::Down => (0, 1, 12, -4),
        }
    }

    // Shift with the next card going into a random (eligible) row/col
    pub fn shift(&self, dir: Direction, next: Option<Card>, rng: &mut RngType) -> Option<Self> {
        let (mut new_grid, new_high_card, shifted_mask) = self.shift_existing(dir);

        // If nothing shifted, this direction can't shift
        if shifted_mask == 0 {
            return None;
        }

        if let Some(next) = next {
            let (outer_start, outer_incr, inner_start, inner_incr) = self.offsets_for(dir);

            // Start by populating a small array with the rows/cols that shifted...
            let mut candidates = [0isize; BOARD_SIZE];
            let mut candidate_count = 0;
            for i in 0..BOARD_SIZE {
                if (shifted_mask >> i) & 1 == 1 {
                    candidates[candidate_count] = outer_start + outer_incr * i as isize;
                    candidate_count += 1;
                }
            }

            // Then pick one row/col at random, and insert the new card there.
            let outer = *candidates[..candidate_count].choose(rng).unwrap();
            let new_spot = (outer + inner_start + inner_incr * 3) as usize;
            new_grid[new_spot] = next;
        }

        Some(Self {
            grid: new_grid,
            high_card: new_high_card,
        })
    }

    // Generate all the possible BoardStates for a given shift direction,
    // i.e. one for each row/col that shifted, because that's where the next card can go,
    // times 1 for each possible next_cards (i.e. bonus cards).
    pub fn shift_all(&self, dir: Direction, next_cards: Vec<u16>) -> Vec<Self> {
        let (new_grid, new_high_card, shifted_mask) = self.shift_existing(dir);

        let mut new_states: Vec<Self> = vec![];

        let (outer_start, outer_incr, inner_start, inner_incr) = self.offsets_for(dir);
        for i in 0..BOARD_SIZE {
            if (shifted_mask >> i) & 1 == 1 {
                let mut new_grid = new_grid.clone();

                let new_spot =
                    (outer_start + outer_incr * i as isize + inner_start + inner_incr * 3) as usize;

                for card in next_cards.iter() {
                    new_grid[new_spot] = *card;

                    new_states.push(Self {
                        grid: new_grid,
                        high_card: new_high_card,
                    });
                }
            }
        }

        new_states
    }

    pub fn shift_mask(&self, dir: Direction) -> u8 {
        let (_, _, shifted_mask) = self.shift_existing(dir);
        shifted_mask
    }

    // Shift all the cards currently on the board
    fn shift_existing(&self, dir: Direction) -> (Grid, Card, u8) {
        let mut new_grid = self.grid.clone();
        let mut new_high_card = self.high_card;

        // Use a mask instead of a HashSet for performance and rng predictability.
        // Bit r set means row/col r had a shift.
        let mut shifted_mask: u8 = 0;

        let (outer_start, outer_incr, inner_start, inner_incr) = self.offsets_for(dir);
        // Loop in one direction
        for outer_round in 0..BOARD_SIZE {
            let outer = outer_start + outer_incr * outer_round as isize;

            // Loop in the perpendicular direction
            for inner_round in 0..BOARD_SIZE - 1 {
                let inner = inner_start + inner_incr * inner_round as isize;

                let cur = (outer + inner) as usize;
                let next = (outer + inner + inner_incr) as usize;

                // Shift as needed, recording shifts in shifted_mask
                if new_grid[cur] == 0 {
                    if new_grid[next] > 0 {
                        shifted_mask |= 1 << outer_round;
                    }

                    new_grid[cur] = new_grid[next];
                    new_grid[next] = 0;
                } else if new_grid[cur] >= 3 {
                    if new_grid[cur] == new_grid[next] {
                        new_grid[cur] *= 2;
                        new_grid[next] = 0;

                        shifted_mask |= 1 << outer_round;
                        if new_grid[cur] > new_high_card {
                            new_high_card = new_grid[cur];
                        }
                    }
                } else if new_grid[cur] + new_grid[next] == 3 {
                    new_grid[cur] = 3;
                    new_grid[next] = 0;

                    shifted_mask |= 1 << outer_round;
                }
            }
        }

        (new_grid, new_high_card, shifted_mask)
    }

    pub fn high_card(&self) -> &Card {
        &self.high_card
    }

    pub fn colorize(cell: Card, cell_as_str: &str) -> StyledContent<&str> {
        match cell {
            1 => cell_as_str.blue(),
            2 => cell_as_str.red(),
            n if n >= 192 => cell_as_str.yellow(),
            _ => cell_as_str.reset(),
        }
    }

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                let cell = self.grid[r * BOARD_SIZE + c];
                let value = match cell {
                    0 => ".".to_string(),
                    _ => cell.to_string(),
                };
                let padded = format!("{value:^5}");
                let styled = BoardState::colorize(cell, &padded);
                write!(f, "{:^4}", styled)?;
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

    use rng_util::test_rng;

    const ARTIFICIAL_NEXT_VALUE: u16 = 4;

    #[test]
    fn initialize() {
        let mut rng = test_rng();

        let mut main1 = DrawPile::initialize(&mut rng);
        let board1 = BoardState::initialize(&mut main1, &mut rng);

        assert_eq!(3, main1.len().0, "draw pile had 9 cards removed");

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
        let mut rng = test_rng();

        let before = [
            0, 3, 0, 3,
            0, 0, 3, 3,
            3, 0, 3, 0,
            3, 6, 0, 3,
        ];
        let after = [
            3, 0, 3, ARTIFICIAL_NEXT_VALUE,
            0, 3, 3, ARTIFICIAL_NEXT_VALUE,
            3, 3, 0, ARTIFICIAL_NEXT_VALUE,
            3, 6, 3, ARTIFICIAL_NEXT_VALUE,
        ];
        test_shift(before, after, &mut rng, "all the basic shift cases, no merges");

        let before = [
            3, 3, 0, 0,
            3, 3, 3, 3,
            3, 6, 12, 24,
            0, 0, 0, 0,
        ];
        let after = [
            6, 0, 0, ARTIFICIAL_NEXT_VALUE,
            6, 3, 3, ARTIFICIAL_NEXT_VALUE,
            3, 6, 12, 24,
            0, 0, 0, 0,
        ];
        test_shift(before, after, &mut rng, "all the basic merge cases");

        let before = [
            1, 1, 0, 0,
            2, 2, 0, 0,
            1, 2, 0, 0,
            2, 1, 0, 0,
        ];
        let after = [
            1, 1, 0, 0,
            2, 2, 0, 0,
            3, 0, 0, ARTIFICIAL_NEXT_VALUE,
            3, 0, 0, ARTIFICIAL_NEXT_VALUE,
        ];
        test_shift(before, after, &mut rng, "1 and 2 are special");

        let before = [
            1, 3, 1, 0,
            2, 3, 2, 0,
            1, 2, 3, 0,
            1, 2, 1, 2,
        ];
        let after = [
            1, 3, 1, 0,
            2, 3, 2, 0,
            3, 3, 0, ARTIFICIAL_NEXT_VALUE,
            3, 1, 2, ARTIFICIAL_NEXT_VALUE,
        ];
        test_shift(before, after, &mut rng, "1 and 2 and 3");

        let before = [
             3,  6, 12, 24,
            24, 12,  6,  3,
             3,  6, 12, 24,
             3,  6,  3,  6,
        ];
        let start_state = BoardState {
            grid: before,
            high_card: 3,
        };
        assert_eq!(
            None, start_state.shift(Direction::Left, None, &mut rng),
            "get a None when nothing can move: left"
        );

        let before = rotate_right(&before);
        let start_state = BoardState {
            grid: before,
            high_card: 3,
        };
        assert_eq!(
            None, start_state.shift(Direction::Up, None, &mut rng),
            "get a None when nothing can move: up"
        );
    }

    fn test_shift(before: Grid, after: Grid, rng: &mut RngType, desc: &str) {
        test_shift_direction(Direction::Left, before, after, rng, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Up, before, after, rng, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Right, before, after, rng, desc);

        let (before, after) = (rotate_right(&before), rotate_right(&after));
        test_shift_direction(Direction::Down, before, after, rng, desc);
    }

    fn test_shift_direction(
        dir: Direction,
        start: Grid,
        expected: Grid,
        rng: &mut RngType,
        desc: &str,
    ) {
        let start_state = BoardState {
            grid: start,
            high_card: 3,
        };
        let expected_state = BoardState {
            grid: expected,
            high_card: 3,
        };

        // first test regular shift()
        let shift_actual_state = start_state
            .shift(dir, Some(ARTIFICIAL_NEXT_VALUE), rng)
            .unwrap();
        let message = format!("{desc}: {dir}, from start state:\n{start_state}\nexpected:\n{expected_state}\nactual:\n{shift_actual_state}");
        compare_states(expected, shift_actual_state.grid, message, 1);

        // then test shift_all()
        let shift_all_actual_states = start_state.shift_all(dir, vec![ARTIFICIAL_NEXT_VALUE]);

        let message_base = format!("{desc}: {dir}, from start state:\n{start_state}\nexpected:\n{expected_state}\nactual:\n");
        let actuals_message = shift_all_actual_states
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n---\n");
        let merge_message = format!("{message_base}{actuals_message}");
        let merged_grid = merge_states(shift_all_actual_states, &merge_message);

        let merged_state = BoardState {
            grid: merged_grid,
            high_card: 3,
        };
        let compare_message = format!("{message_base}{merged_state}");
        compare_states(
            expected,
            merged_grid,
            compare_message,
            expected
                .iter()
                .filter(|&card| *card == ARTIFICIAL_NEXT_VALUE)
                .count(),
        );
    }

    fn compare_states(expected: Grid, actual: Grid, message: String, expected_nexts: usize) {
        let mut next_seen = 0;
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                let expected = expected[r * BOARD_SIZE + c];
                let actual = actual[r * BOARD_SIZE + c];
                if ARTIFICIAL_NEXT_VALUE == expected {
                    if ARTIFICIAL_NEXT_VALUE == actual {
                        next_seen += 1;
                    }
                } else {
                    if expected != actual {
                        panic!("{message}");
                    }
                }
            }
        }
        assert_eq!(
            expected_nexts, next_seen,
            "'Next' values (i.e. {ARTIFICIAL_NEXT_VALUE}s) were where we expected: {message}"
        );
    }

    fn merge_states(all_states: Vec<BoardState>, message: &String) -> Grid {
        match all_states.len() {
            0 => panic!("No BoardStates in all_states; this shouldn't happen: {message}"),
            1 => all_states[0].grid,
            _ => {
                let mut merged_grid = all_states[0].grid;
                for i in 1..all_states.len() {
                    for r in 0..BOARD_SIZE {
                        for c in 0..BOARD_SIZE {
                            let merged_val = merged_grid[r * BOARD_SIZE + c];
                            let next_val = all_states[i].grid[r * BOARD_SIZE + c];
                            if ARTIFICIAL_NEXT_VALUE == next_val {
                                merged_grid[r * BOARD_SIZE + c] = ARTIFICIAL_NEXT_VALUE;
                            } else if ARTIFICIAL_NEXT_VALUE == merged_val && 0 == next_val {
                                // do nothing
                            } else {
                                assert_eq!(
                                    merged_val, next_val,
                                    "unexpected value in state #{i} at r {r} c {c} when merging all states: {message}"
                                );
                            }
                        }
                    }
                }
                merged_grid
            }
        }
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
