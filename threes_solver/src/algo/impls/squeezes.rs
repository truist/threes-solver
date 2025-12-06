use threes_simulator::game_state::{Card, Direction, GameState};

use crate::algo::core::Algo;

use super::super::core::ValueBooster;
use super::super::neighbors::iterate_with_neighbors;

// a smaller card "squeezed" between bigger cards and/or the wall
#[derive(Debug)]
pub(crate) struct Squeezes;

impl Algo for Squeezes {
    fn score(&self, game_state: &GameState, booster: Option<&dyn ValueBooster>) -> f64 {
        let mut score = 0.0;

        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            for pair in [
                (
                    neighbors[Direction::Left as usize],
                    neighbors[Direction::Right as usize],
                ),
                (
                    neighbors[Direction::Up as usize],
                    neighbors[Direction::Down as usize],
                ),
            ] {
                if self.is_pair_squeezing(card, pair) {
                    if let Some(booster) = booster {
                        score += booster.boost_score_for(1.0, &[card])
                    } else {
                        score += 1.0;
                    }
                }
            }
        });

        score
    }

    fn normalization_factor(&self) -> f64 {
        super::ALGO_MAX_BASE / 16.0
    }
}

impl Squeezes {
    fn is_pair_squeezing(&self, middle: Card, pair: (Card, Card)) -> bool {
        let (m, (a, b)) = (middle, pair);

        // this 'cleverly' takes advantage of wall-side "neighbors" being Card::MAX
        m > 0 && a > 2 && a > m && b > 2 && b > m
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::super::test_utils::generate_game_state;
    use super::super::super::wrappers::value_booster_wrapper::ValueBoosterWrapper;

    #[test]
    #[rustfmt::skip]
    fn test_squeezes() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "no squeezes when everything is mergeable"
        );

        let mut grid = [0; 16];
        grid[5] = 3;
        let game_state = generate_game_state(grid);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "no squeezes when just one cell filled"
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "no squeezes when cards don't have neighbors"
        );

        let game_state = generate_game_state([
            6, 3, 3, 6,
            3, 0, 0, 3,
            3, 0, 0, 3,
            6, 3, 3, 6,
        ]);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "no squeezes when squeezed cards are mergeable"
        );

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "1s and 2s can't squeeze each other"
        );

        let game_state = generate_game_state([
            3, 2, 6, 0,
            0, 0, 0, 0,
            3, 1, 6, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(
            2.0,
            Squeezes.score(&game_state, None),
            "1s and 2s can be squeezed by >= 3"
        );

        let game_state = generate_game_state([
            24, 12, 6, 0,
            12,  6, 3, 0,
             6,  3, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            0.0,
            Squeezes.score(&game_state, None),
            "no squeezes when bigger neighbors aren't on opposite sides"
        );

        let game_state = generate_game_state([
            6, 3, 6, 0,
            3, 0, 0, 6,
            6, 0, 0, 3,
            0, 6, 3, 6,
        ]);
        assert_eq!(
            4.0,
            Squeezes.score(&game_state, None),
            "obvious squeezes in both directions"
        );

        let game_state = generate_game_state([
            3, 6, 0, 3,
            0, 0, 0, 6,
            6, 0, 0, 0,
            3, 0, 6, 3,
        ]);
        assert_eq!(
            4.0,
            Squeezes.score(&game_state, None),
            "you can be squeezed between a card and the wall, in all four directions"
        );

        let game_state = generate_game_state([
            24, 12, 6, 3,
             0,  0, 0, 0,
             0,  0, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            1.0,
            Squeezes.score(&game_state, None),
            "the three is squeezed but I'm not sure it should be"
            // TODO handle this case better?
        );

        let game_state = generate_game_state([
            0, 0, 6, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(
            1.0,
            Squeezes.score(&game_state, None),
            "the three counts as squeezed but that seems wrong"
            // TODO handle this case better?
        );

        let game_state = generate_game_state([
            24, 6, 12,  3, // 2
            12, 6, 24,  6, // 2
             3, 3,  6,  3, // 1
             3, 0,  6, 12, // 0
         //  0  0   1   2
        ]);
        assert_eq!(
            (2 + 2 + 1 + 0 + 0 + 0 + 1 + 2) as f64,
            Squeezes.score(&game_state, None),
            "a big complex example"
        );

        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: Box::new(Squeezes),
            min_value_to_boost: 1,
            max_value_to_boost: 6,
            boost: test_boost,
        };
        let game_state = generate_game_state([
            12,  2, 24,  0, // 2 is squeezed and boosted (1 * boost)
            12,  1,  6,  0, // 1 is squeezed and boosted (1 * boost)
             6, 12, 24,  6, // 6s are squeezed and boosted (2 * boost)
            24, 24, 12, 24, // 12 is squeezed but not boosted (1)
         //  ^- 6 is squeezed and boosted (1 * boost)
         //      ^- 1 can't be squeezed by 2; 12 isn't squeezed (0)
         //          ^- 12 is squeezed but not boosted (1); 6 is squeezed and boosted (1 * boost)
         //              ^- no squeezes (0)
        ]);
        assert_eq!(
            1.0 * test_boost + 1.0 * test_boost + 2.0 * test_boost + 1.0
                + 1.0 * test_boost + 0.0 + 1.0 + 1.0 * test_boost + 0.0,
            Squeezes.score(&game_state, Some(&booster)),
            "a big complex boosted example, covering a bunch of cases"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_max_possible_score() {
        let game_state = generate_game_state([
            3, 3, 3, 3,
            6, 6, 6, 6,
            6, 6, 6, 6,
            3, 3, 3, 3,
        ]);
        assert_eq!(8.0, Squeezes.score(&game_state, None), "squeezes max score, first case");

        // 16
        let game_state = generate_game_state([
            3, 6, 3, 6,
            6, 3, 6, 3,
            3, 6, 3, 6,
            6, 3, 6, 3,
        ]);
        assert_eq!(8.0 * 2.0, Squeezes.score(&game_state, None), "squeezes max score, second case");

        let game_state = generate_game_state([
            3, 6, 3, 6,
            3, 6, 3, 6,
            3, 6, 3, 6,
            3, 6, 3, 6,
        ]);
        assert_eq!(8.0, Squeezes.score(&game_state, None), "squeezes max score, third case");

        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 6, 6, 3,
            3, 6, 6, 3,
            3, 3, 3, 3,
        ]);
        assert_eq!(8.0, Squeezes.score(&game_state, None), "squeezes max score, fourth case");
    }
}
