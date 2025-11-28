use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::Algos;

use super::super::core::AlgoScalers;
use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // cards that can merge with each other
    pub(crate) fn merges(&self, game_state: &GameState, scalers: &AlgoScalers) -> f64 {
        self.merge_impl(false, game_state, scalers)
    }

    // cards that are one off from merging with each other (e.g. 3 and 6)
    pub(crate) fn nearly_merges(&self, game_state: &GameState, scalers: &AlgoScalers) -> f64 {
        self.merge_impl(true, game_state, scalers)
    }

    fn merge_impl(&self, nearly_mode: bool, game_state: &GameState, scalers: &AlgoScalers) -> f64 {
        let mut count = 0.0;
        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            for neighbor in neighbors.iter() {
                let matches = if nearly_mode {
                    self.are_nearly_mergable(card, *neighbor)
                } else {
                    self.can_merge(card, *neighbor)
                };
                if matches {
                    count += scalers.scale_score(1.0, game_state, &[card, *neighbor]);
                }
            }
        });
        count / 2.0
    }

    fn can_merge(&self, left: Card, right: Card) -> bool {
        left > 0
            && right > 0
            && left < Card::MAX
            && right < Card::MAX
            && (left + right == 3 || (left > 2 && left == right))
    }

    fn are_nearly_mergable(&self, left: Card, right: Card) -> bool {
        // 1 with 3
        // 2 with 3
        // anything else with 2x itself or 0.5x itself
        left > 0
            && right > 0
            && left < Card::MAX
            && right < Card::MAX
            && ((left < 3 && right == 3 || left == 3 && right < 3)
                || (left >= 3 && right >= 3 && (left == right * 2 || left * 2 == right)))
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::AlgoScalers;
    use crate::algo::core::Algos::{Merges, NearlyMerges};

    use super::super::super::test_utils::generate_game_state;
    use super::super::super::wrappers::ValueScaler;

    #[test]
    #[rustfmt::skip]
    fn test_merges() {
        let no_scalers = &AlgoScalers {
            scalers: vec![],
        };
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0,
            Merges.merges(&game_state, no_scalers),
            "max score when everything is mergeable",
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(0.0, Merges.merges(&game_state, no_scalers), "no merges gives a score of 0");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1.0, Merges.merges(&game_state, no_scalers), "1 merge gives a score of 1");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(4.0, Merges.merges(&game_state, no_scalers), "4 pairs of merges in a 2x2");

        let game_state = generate_game_state([
            1, 2, 2, 0,
            0, 2, 2, 1,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ]);
        assert_eq!(2.0, Merges.merges(&game_state, no_scalers), "1's and 2's only merge with their counterpart");

        let game_state = generate_game_state([
            3, 3, 2, 1,  // 2
            0, 3, 6, 6,  // 1
            2, 1, 0, 12, // 1
            1, 2, 1, 2,  // 3  <-- this one was surprising
         // 1  2  0  0
        ]);
        assert_eq!(
            (2 + 1 + 1 + 3 + 1 + 2 + 0 + 0) as f64,
            Merges.merges(&game_state, no_scalers),
            "a big messy example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_scaled_merges() {
        let test_scale = 3.0;
        let scaler = ValueScaler {
            min_value_to_scale: 1,
            max_value_to_scale: 6,
            scale: test_scale,
        };
        let scalers = &AlgoScalers {
            scalers: vec![&scaler],
        };

        let game_state = generate_game_state([12; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0,
            Merges.merges(&game_state, scalers),
            "regular score when nothing matches the scaler",
        );

        let game_state = generate_game_state([6; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0 * test_scale,
            Merges.merges(&game_state, scalers),
            "scaled score when everything matches the scaler",
        );

        let game_state = generate_game_state([
            12, 12, 0, 0,
            12, 12, 0, 0,
             0,  0, 6, 6,
             0,  0, 6, 6,
        ]);
        assert_eq!(4.0, Merges.merges(&game_state, scalers), "12s are ignored, 6s aren't");

        let scaler = ValueScaler {
            min_value_to_scale: 1,
            max_value_to_scale: 1,
            scale: test_scale,
        };
        let scalers = &AlgoScalers {
            scalers: vec![&scaler],
        };
        let game_state = generate_game_state([
            3, 3, 6, 6,
            0, 0, 0, 0,
            1, 2, 3, 6, // 1/2 has a merge and 1 matches the scaler
            0, 0, 0, 0,
        ]);
        assert_eq!(
            1.0 * test_scale, Merges.merges(&game_state, scalers),
            "1s can merge with 2s if either is matched by the scaler"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_nearly_merges() {
        let no_scalers = &AlgoScalers {
            scalers: vec![],
        };

        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(0.0, NearlyMerges.nearly_merges(&game_state, no_scalers), "0 when everything is {value}");
        }

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(0.0, NearlyMerges.nearly_merges(&game_state, no_scalers), "1s and 2s aren't nearly mergeable");

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(4.0, NearlyMerges.nearly_merges(&game_state, no_scalers), "1s and 2s merge with 3s");

        let game_state = generate_game_state([
             3, 6, 3, 12, // 2
             6, 0, 3,  0, // 0
            12, 3, 0, 12, // 0
             0, 0, 0, 24, // 0
        //   2  0  0   1
        ]);
        assert_eq!(
            5.0, NearlyMerges.nearly_merges(&game_state, no_scalers),
            "Cards merge with cards twice their value"
        );

        let game_state = generate_game_state([
            1, 6, 3,  2, // 2
            6, 0, 3,  0, // 0
            2, 3, 0, 12, // 1
            0, 0, 0, 24, // 0
        //  0  0  0   1
        ]);
        assert_eq!(4.0, NearlyMerges.nearly_merges(&game_state, no_scalers), "A mix of everything");
    }

    #[test]
    #[rustfmt::skip]
    fn test_scaled_nearly_merges() {
        let test_scale = 3.0;
        let scaler = ValueScaler {
            min_value_to_scale: 1,
            max_value_to_scale: 6,
            scale: test_scale,
        };
        let scalers = &AlgoScalers {
            scalers: vec![&scaler],
        };

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(
            0.0, NearlyMerges.nearly_merges(&game_state, scalers),
            "Even though they match the scaler, 1s and 2s aren't nearly mergeable"
        );

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(
            4.0 * test_scale, NearlyMerges.nearly_merges(&game_state, scalers),
            "1s and 2s match the scaler and merge with 3s"
        );

        let game_state = generate_game_state([
            24, 12, 6, 3,
             0,  0, 6, 0,
             0,  0, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            2.0 * test_scale, NearlyMerges.nearly_merges(&game_state, scalers),
            "6 passes the scaler and is nearly mergeable with the values on either side of it"
        );

    }
}
