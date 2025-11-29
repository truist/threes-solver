use threes_simulator::board_state::BOARD_SIZE;
use threes_simulator::game_state::GameState;

use crate::algo::core::Algos;

use super::super::core::ValueBooster;
use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // Higher values near a wall.
    pub(crate) fn high_walls(
        &self,
        game_state: &GameState,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
        self.high_impl(game_state, false, booster)
    }

    // Higher values in a corner.
    pub(crate) fn high_corners(
        &self,
        game_state: &GameState,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
        self.high_impl(game_state, true, booster)
    }

    // Specifically, find the top three *actual* highest values on the board,
    // then give a point for each card with one of those values next to a wall or in a corner.
    // A card in a corner will only be counted once.
    // Give 1 point for the lowest of the three, 2 for the middle, and 3 for the highest.
    fn high_impl(
        &self,
        game_state: &GameState,
        corners_only: bool,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
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

        let mut score = 0.0;
        iterate_with_neighbors(grid, |index, card, _neighbors| {
            let mut count_cell = false;
            if card > 0 {
                if corners_only {
                    count_cell = index == 0                              // top left
                        || index == BOARD_SIZE - 1                       // top right
                        || index == BOARD_SIZE * BOARD_SIZE - BOARD_SIZE // bottom left
                        || index == BOARD_SIZE * BOARD_SIZE - 1; // bottom right
                } else {
                    count_cell = index % BOARD_SIZE == 0                  // left wall
                        || index < BOARD_SIZE                             // top wall
                        || (index + 1) % BOARD_SIZE == 0                  // right wall
                        || index >= BOARD_SIZE * BOARD_SIZE - BOARD_SIZE; // bottom wall
                }
            }

            if count_cell {
                let score_increment = if card == great_big {
                    3.0
                } else if card == middle {
                    2.0
                } else if card == wee {
                    1.0
                } else {
                    0.0
                };

                if let Some(booster) = booster {
                    score += booster.boost_score_for(score_increment, &[card]);
                } else {
                    score += score_increment;
                }
            }
        });

        score
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos::{HighCorner, HighWall};

    use super::super::super::test_utils::generate_game_state;
    use super::super::super::wrappers::ValueBoosterWrapper;

    #[test]
    #[rustfmt::skip]
    fn test_high_walls() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(
                12.0 * 3.0, HighWall.high_walls(&game_state, None),
                "36 when all cells have the same value"
            );
        }

        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 0, 0, 3,
            3, 0, 0, 3,
            3, 3, 3, 3,
        ]);
        assert_eq!(
            12.0 * 3.0, HighWall.high_walls(&game_state, None),
            "double check that all wall spots are counted"
        );

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(0.0, HighWall.high_walls(&game_state, None), "0 when no values are near a wall");

        let game_state = generate_game_state([
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3.0, HighWall.high_walls(&game_state, None), "3 when the highest high value is near a wall");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3.0, HighWall.high_walls(&game_state, None), "still 3 when the highest value is in a corner");

        let game_state = generate_game_state([
            3,  3,  0, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(
            0.0, HighWall.high_walls(&game_state, None),
            "0 when only non-highest values are near the walls"
        );

        let game_state = generate_game_state([
            3,  3, 24, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(
            2.0, HighWall.high_walls(&game_state, None),
            "2 when medium-high value is next to a wall, and other high values aren't"
        );

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(
            4.0, HighWall.high_walls(&game_state, None),
            "The top 3 *actual* high values are used, even if there's a gap"
        );

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            1,  2, 48, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(3.0, HighWall.high_walls(&game_state, None), "A more complex example");

        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: HighWall,
            min_value_to_boost: 12,
            max_value_to_boost: 48,
            boost: test_boost,
        };
        let game_state = generate_game_state([
            12, 96, 48, 0, // 96 is great_big (3); 48 is boosted (2 * boost)
            12, 12, 24, 0, // 12 isn't high (0)
             1,  2, 48, 0, // 48 not on a wall doesn't count (0)
             0, 24,  0, 0, // 24 is high and boosted (wee: 1 * boost)
        ]);
        assert_eq!(
            3.0 + 2.0 * test_boost + 1.0 * test_boost, HighWall.high_walls(&game_state, Some(&booster)),
            "A booster example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_high_corners() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(
                4.0 * 3.0, HighCorner.high_corners(&game_state, None),
                "12 when all cells have the same value"
            );
        }

        let game_state = generate_game_state([
            3, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            3, 0, 0, 3,
        ]);
        assert_eq!(
            4.0 * 3.0, HighCorner.high_corners(&game_state, None),
            "double check that all corner spots are counted"
        );

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 1,
            0, 0, 0, 0,
        ]);
        assert_eq!(0.0, HighCorner.high_corners(&game_state, None), "0 when no values are in a corner");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(
            3.0, HighCorner.high_corners(&game_state, None),
            "3 when the highest high value is in a corner"
        );

        let game_state = generate_game_state([
            3,  3,  0, 3,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0, 48,  0, 0,
        ]);
        assert_eq!(
            0.0, HighCorner.high_corners(&game_state, None),
            "0 when only non-highest values are in the corners"
        );

        let game_state = generate_game_state([
            24, 3,  3, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 3,
        ]);
        assert_eq!(
            2.0, HighCorner.high_corners(&game_state, None),
            "2 when medium-high value is in a corner, and other high values aren't"
        );

        let game_state = generate_game_state([
            96, 3, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            12, 0,  0, 0,
        ]);
        assert_eq!(
            4.0, HighCorner.high_corners(&game_state, None),
            "The top 3 *actual* high values are used, even if there's a gap"
        );

        let game_state = generate_game_state([
            24, 96, 3, 96,
            3,  3, 24,  0,
            1,  2, 48,  0,
            48, 0,  0, 24,
        ]);
        assert_eq!(7.0, HighCorner.high_corners(&game_state, None), "A more complex example");

        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: HighCorner,
            min_value_to_boost: 12,
            max_value_to_boost: 24,
            boost: test_boost,
        };
        let game_state = generate_game_state([
            48, 96,  3, 96, // 48 (mid: 2) and 96 (high: 3) aren't boosted
             3,  3, 24,  0, // no corners
             1,  2, 48,  0, // no corners
            24,  0,  0, 12, // 24 is boosted (wee: 1 * boost); 12 is boosted but isn't high
        ]);
        assert_eq!(
            2.0 + 3.0 + 1.0 * test_boost, HighCorner.high_corners(&game_state, Some(&booster)),
            "A booster example"
        );
    }
}
