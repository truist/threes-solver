use threes_simulator::board_state::BOARD_SIZE;
use threes_simulator::game_state::GameState;

use super::super::neighbors::iterate_with_neighbors;

// Higher values near a wall.
pub(crate) fn high_walls(game_state: &GameState) -> u8 {
    high_impl(game_state, false)
}

// Higher values in a corner.
pub(crate) fn high_corners(game_state: &GameState) -> u8 {
    high_impl(game_state, true)
}

// Specifically, find the top three *actual* highest values on the board,
// then give a point for each card with one of those values next to a wall or in a corner.
// A card in a corner will only be counted once.
// Give 1 point for the lowest of the three, 2 for the middle, and 3 for the highest.
#[rustfmt::skip]
fn high_impl(game_state: &GameState, corners_only: bool) -> u8 {
    let grid = game_state.get_grid();

    // https://americanliterature.com/childrens-stories/goldilocks-and-the-three-bears
    let (mut great_big, mut middle, mut wee) = (0, 0, 0);
    for card in grid.iter() {
        if *card > great_big {
            wee = middle;
            middle = great_big;
            great_big = *card;
        } else if *card < great_big && *card > middle {
            wee = middle;
            middle = *card;
        } else if *card < middle && *card > wee {
            wee = *card;
        }
    }

    let mut score = 0;
    iterate_with_neighbors(grid, |index, card, _neighbors| {
        let mut count_cell = false;
        if card > 0 {
            if corners_only {
                count_cell = index == 0                              // top left
                    || index == BOARD_SIZE - 1                       // top right
                    || index == BOARD_SIZE * BOARD_SIZE - BOARD_SIZE // bottom left
                    || index == BOARD_SIZE * BOARD_SIZE - 1;         // bottom right
            } else {
                count_cell = index % BOARD_SIZE == 0                  // left wall
                    || index < BOARD_SIZE                             // top wall
                    || (index + 1) % BOARD_SIZE == 0                  // right wall
                    || index >= BOARD_SIZE * BOARD_SIZE - BOARD_SIZE; // bottom wall
            }
        }

        if count_cell {
            if card == great_big {
                score += 3;
            } else if card == middle {
                score += 2;
            } else if card == wee {
                score += 1;
            }
        }
    });

    score as u8
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_high_walls() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(12 * 3, high_walls(&game_state), "36 when all cells have the same value");
        }

        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 0, 0, 3,
            3, 0, 0, 3,
            3, 3, 3, 3,
        ]);
        assert_eq!(12 * 3, high_walls(&game_state), "double check that all wall spots are counted");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(0, high_walls(&game_state), "0 when no values are near a wall");

        let game_state = generate_game_state([
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "3 when the highest high value is near a wall");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "still 3 when the highest value is in a corner");

        let game_state = generate_game_state([
            3,  3,  0, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(0, high_walls(&game_state), "0 when only non-highest values are near the walls");

        let game_state = generate_game_state([
            3,  3, 24, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(2, high_walls(&game_state), "2 when medium-high value is next to a wall, and other high values aren't");

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(4, high_walls(&game_state), "The top 3 *actual* high values are used, even if there's a gap");

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            1,  2, 48, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "A more complex example");
    }

    #[test]
    #[rustfmt::skip]
    fn test_high_corners() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(4 * 3, high_corners(&game_state), "12 when all cells have the same value");
        }

        let game_state = generate_game_state([
            3, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            3, 0, 0, 3,
        ]);
        assert_eq!(4 * 3, high_corners(&game_state), "double check that all corner spots are counted");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 1,
            0, 0, 0, 0,
        ]);
        assert_eq!(0, high_corners(&game_state), "0 when no values are in a corner");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_corners(&game_state), "3 when the highest high value is in a corner");

        let game_state = generate_game_state([
            3,  3,  0, 3,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0, 48,  0, 0,
        ]);
        assert_eq!(0, high_corners(&game_state), "0 when only non-highest values are in the corners");

        let game_state = generate_game_state([
            24, 3,  3, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 3,
        ]);
        assert_eq!(2, high_corners(&game_state), "2 when medium-high value is in a corner, and other high values aren't");

        let game_state = generate_game_state([
            96, 3, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            12, 0,  0, 0,
        ]);
        assert_eq!(4, high_corners(&game_state), "The top 3 *actual* high values are used, even if there's a gap");

        let game_state = generate_game_state([
            24, 96, 3, 96,
            3,  3, 24,  0,
            1,  2, 48,  0,
            48, 0,  0, 24,
        ]);
        assert_eq!(7, high_corners(&game_state), "A more complex example");
    }
}
