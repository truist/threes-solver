use threes_simulator::game_state::{Card, Direction, GameState};

use crate::algo::core::Algos;

use super::super::core::ValueFilter;
use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // a smaller card "squeezed" between bigger cards and/or the wall
    pub(crate) fn squeezes(&self, game_state: &GameState, filter: Option<&dyn ValueFilter>) -> f64 {
        let mut count = 0.0;

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
                if self.is_pair_squeezing(card, pair)
                    && filter.is_none_or(|filter| filter.filter_values(&[card]))
                {
                    count += 1.0;
                }
            }
        });

        count
    }

    fn is_pair_squeezing(&self, middle: Card, pair: (Card, Card)) -> bool {
        let (m, (a, b)) = (middle, pair);

        // this 'cleverly' takes advantage of wall-side "neighbors" being Card::MAX
        m > 0 && a > 2 && a > m && b > 2 && b > m
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos::Squeezes;

    use super::super::super::test_utils::generate_game_state;
    use super::super::super::wrappers::ValueFilterWrapper;

    #[test]
    #[rustfmt::skip]
    fn test_squeezes() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            0.0,
            Squeezes.squeezes(&game_state, None),
            "no squeezes when everything is mergeable"
        );

        let mut grid = [0; 16];
        grid[5] = 3;
        let game_state = generate_game_state(grid);
        assert_eq!(
            0.0,
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
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
            Squeezes.squeezes(&game_state, None),
            "a big complex example"
        );

        let filter = ValueFilterWrapper {
            wrapped: Squeezes,
            min_value_to_keep: 1,
            max_value_to_keep: 6,
        };
        let game_state = generate_game_state([
            12,  2, 24,  0, // 2 is kept and squeezed (1)
            12,  1,  6,  0, // 1 is kept and squeezed (1)
             6, 12, 24,  6, // 6s are kept and squeezed (2)
            24, 12, 12, 12, // 12s are squeezed but not kept (0)
         //  ^- 6 is squeezed (1)
         //      ^- 1 can't be squeezed by 2; 12 isn't squeezed (0)
         //          ^- 12 is squeezed but not kept; 6 is squeezed and kept (1)
         //              ^- no squeezes (0)
        ]);
        assert_eq!(
            (1 + 1 + 2 + 0 + 1 + 0 + 1 + 0) as f64,
            Squeezes.squeezes(&game_state, Some(&filter)),
            "a big complex filtered example, covering a bunch of cases"
        );
    }
}
