use threes_simulator::game_state::{Card, Direction, GameState};

use super::super::neighbors::iterate_with_neighbors;

// Add points when left->right is monotonically increasing or decreasing, per row.
// Ditto for up->down, per col.
// Subtract points when they aren't.
// It's OK for neighboring rows (or cols) to run in opposite directions.
// 1s and 2s are treated as distinct values.
pub(crate) fn monotones(game_state: &GameState) -> u8 {
    let mut score: i8 = 0;

    iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
        let right = neighbors[Direction::Right as usize];
        if right < Card::MAX {
            if right > card {
                score += 1;
            } else if right < card {
                score -= 1;
            }
        }

        let down = neighbors[Direction::Down as usize];
        if down < Card::MAX {
            if down > card {
                score += 1;
            } else if down < card {
                score -= 1;
            }
        }
    });

    score.abs() as u8
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_monotones() {
        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(0, monotones(&game_state), "0 when all cells have the same value {value}");
        }

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 2, 3, 6,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, monotones(&game_state), "a basic monotone row");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 3, 2, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, monotones(&game_state), "it works right to left, too");

        let game_state = generate_game_state([
            0, 1, 0, 0,
            0, 2, 0, 0,
            0, 3, 0, 0,
            0, 6, 0, 0,
        ]);
        assert_eq!(3, monotones(&game_state), "a basic monotone col");

        let game_state = generate_game_state([
            0, 0, 6, 0,
            0, 0, 3, 0,
            0, 0, 2, 0,
            0, 0, 1, 0,
        ]);
        assert_eq!(3, monotones(&game_state), "it works down to up, too");

        let game_state = generate_game_state([
            6, 3, 2, 1, // -3
            3, 0, 0, 0, // -1
            2, 0, 0, 0, // -1
            1, 0, 0, 0, // -1
        // -3 -1 -1 -1
        ]);
        assert_eq!(6 + 6, monotones(&game_state), "both directions at the same time");

        let game_state = generate_game_state([
            0, 0, 0, 1, // 1
            0, 0, 0, 2, // 1
            0, 0, 0, 3, // 1
            1, 2, 3, 6, // 3
        //  1  1  1  3
        ]);
        assert_eq!(12, monotones(&game_state), "the other both directions at the same time");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 3, 2, 6,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1, monotones(&game_state), "non-monotones don't score as well");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 2, 3, 6,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1, monotones(&game_state), "non-monotones don't score as well, the other way");

        let game_state = generate_game_state([
            0, 3, 2, 0, // 1
            0, 3, 2, 0, // 1
            0, 6, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  0  0  1  0
        ]);
        assert_eq!(3, monotones(&game_state), "weird behavior in the middle of the board, but that's probably OK in practice");

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            6, 3, 2, 1, // 3
            0, 0, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  1  1
        ]);
        assert_eq!(6 + 4, monotones(&game_state), "a pretty-good state");

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
            0, 0, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  2  2  0  0
        ]);
        assert_eq!(0 + 4, monotones(&game_state), "a lower but still positive score, with rows alternating directions, which seems reasonable");

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            0, 0, 0, 0, // 0
            1, 2, 3, 6, // -3
            0, 0, 0, 0, // 0
        //  -1 -1 -1 -1
        ]);
        assert_eq!(0 + 4, monotones(&game_state), "interesting to see how it works out with gaps between rows");

        let game_state = generate_game_state([
            48, 24, 12, 6, // 3
            24, 12,  6, 3, // 3
            12,  6,  3, 2, // 3
             6,  3,  2, 1, // 3
        //   3   3   3  3
        ]);
        assert_eq!(3 * 8, monotones(&game_state), "the best possible state for this algo");

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
        // -1 -1  1  1
        ]);
        assert_eq!(0 + 0, monotones(&game_state), "fully back-and-forth screws you, which is maybe reasonable");

        let game_state = generate_game_state([
            24, 12,  2, 3, // -1
            12,  1,  3, 1, // -1
             2, 48, 24, 3, // -1
             2,  6,  3, 1, // -1
        //  -2  -1   1 -1
        ]);
        assert_eq!(4 + 3, monotones(&game_state), "a complex (dead-end) case");
    }
}
