use threes_simulator::game_state::{Card, Direction, GameState};

use crate::algo::core::{AlgoScalers, Algos};

use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // Add points when left->right is monotonically increasing or decreasing, per row.
    // Ditto for up->down, per col.
    // Subtract points when they aren't.
    // It's OK for neighboring rows (or cols) to run in opposite directions.
    // 1s and 2s are treated as distinct values.
    pub(crate) fn monotones(&self, game_state: &GameState, scalers: &AlgoScalers) -> f64 {
        let mut score: f64 = 0.0;

        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            let mut card_score = 0.0;

            let right = neighbors[Direction::Right as usize];
            if right < Card::MAX {
                if right > card {
                    card_score += 1.0;
                } else if right < card {
                    card_score -= 1.0;
                }
            }

            let down = neighbors[Direction::Down as usize];
            if down < Card::MAX {
                if down > card {
                    card_score += 1.0;
                } else if down < card {
                    card_score -= 1.0;
                }
            }

            score += scalers.scale_score(card_score, game_state, &[card]);
        });

        score.abs()
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::AlgoScalers;
    use crate::algo::core::Algos::Monotones;

    use super::super::super::test_utils::generate_game_state;

    #[test]
    #[rustfmt::skip]
    fn test_monotones() {
        let no_scalers = &AlgoScalers {
            scalers: vec![],
        };

        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(
                0.0, Monotones.monotones(&game_state, no_scalers),
                "0 when all cells have the same value {value}"
            );
        }

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 2, 3, 6,
            0, 0, 0, 0,
        ]);
        assert_eq!(3.0, Monotones.monotones(&game_state, no_scalers), "a basic monotone row");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 3, 2, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3.0, Monotones.monotones(&game_state, no_scalers), "it works right to left, too");

        let game_state = generate_game_state([
            0, 1, 0, 0,
            0, 2, 0, 0,
            0, 3, 0, 0,
            0, 6, 0, 0,
        ]);
        assert_eq!(3.0, Monotones.monotones(&game_state, no_scalers), "a basic monotone col");

        let game_state = generate_game_state([
            0, 0, 6, 0,
            0, 0, 3, 0,
            0, 0, 2, 0,
            0, 0, 1, 0,
        ]);
        assert_eq!(3.0, Monotones.monotones(&game_state, no_scalers), "it works down to up, too");

        let game_state = generate_game_state([
            6, 3, 2, 1, // -3
            3, 0, 0, 0, // -1
            2, 0, 0, 0, // -1
            1, 0, 0, 0, // -1
        // -3 -1 -1 -1
        ]);
        assert_eq!(6.0 + 6.0, Monotones.monotones(&game_state, no_scalers), "both directions at the same time");

        let game_state = generate_game_state([
            0, 0, 0, 1, // 1
            0, 0, 0, 2, // 1
            0, 0, 0, 3, // 1
            1, 2, 3, 6, // 3
        //  1  1  1  3
        ]);
        assert_eq!(
            12.0, Monotones.monotones(&game_state, no_scalers),
            "the other both directions at the same time"
        );

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 3, 2, 6,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1.0, Monotones.monotones(&game_state, no_scalers), "non-monotones don't score as well");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            6, 2, 3, 6,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(
            1.0, Monotones.monotones(&game_state, no_scalers),
            "non-monotones don't score as well, the other way"
        );

        let game_state = generate_game_state([
            0, 3, 2, 0, // 1
            0, 3, 2, 0, // 1
            0, 6, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  0  0  1  0
        ]);
        assert_eq!(
            3.0, Monotones.monotones(&game_state, no_scalers),
            "weird behavior in the middle of the board, but that's probably OK in practice"
        );

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            6, 3, 2, 1, // 3
            0, 0, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  1  1
        ]);
        assert_eq!(6.0 + 4.0, Monotones.monotones(&game_state, no_scalers), "a pretty-good state");

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
            0, 0, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  2  2  0  0
        ]);
        assert_eq!(
            0.0 + 4.0, Monotones.monotones(&game_state, no_scalers),
            "a lower but still positive score, with rows alternating directions, which seems reasonable"
        );

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            0, 0, 0, 0, // 0
            1, 2, 3, 6, // -3
            0, 0, 0, 0, // 0
        //  -1 -1 -1 -1
        ]);
        assert_eq!(
            0.0 + 4.0, Monotones.monotones(&game_state, no_scalers),
            "interesting to see how it works out with gaps between rows"
        );

        let game_state = generate_game_state([
            48, 24, 12, 6, // 3
            24, 12,  6, 3, // 3
            12,  6,  3, 2, // 3
             6,  3,  2, 1, // 3
        //   3   3   3  3
        ]);
        assert_eq!(
            3.0 * 8.0, Monotones.monotones(&game_state, no_scalers),
            "the best possible state for this algo"
        );

        let game_state = generate_game_state([
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
            6, 3, 2, 1, // 3
            1, 2, 3, 6, // -3
        // -1 -1  1  1
        ]);
        assert_eq!(
            0.0 + 0.0, Monotones.monotones(&game_state, no_scalers),
            "fully back-and-forth screws you, which is maybe reasonable"
        );

        let game_state = generate_game_state([
            24, 12,  2, 3, // -1
            12,  1,  3, 1, // -1
             2, 48, 24, 3, // -1
             2,  6,  3, 1, // -1
        //  -2  -1   1 -1
        ]);
        assert_eq!(4.0 + 3.0, Monotones.monotones(&game_state, no_scalers), "a complex (dead-end) case");
    }

    // TODO: tests for scalers
}
