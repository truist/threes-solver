use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::Algos;

use super::super::core::ValueFilter;
use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // cards that can merge with each other
    pub(crate) fn merges(&self, game_state: &GameState, filter: Option<&dyn ValueFilter>) -> f64 {
        self.merge_impl(false, game_state, filter)
    }

    // cards that are one off from merging with each other (e.g. 3 and 6)
    pub(crate) fn nearly_merges(
        &self,
        game_state: &GameState,
        filter: Option<&dyn ValueFilter>,
    ) -> f64 {
        self.merge_impl(true, game_state, filter)
    }

    fn merge_impl(
        &self,
        nearly_merge: bool,
        game_state: &GameState,
        filter: Option<&dyn ValueFilter>,
    ) -> f64 {
        let mut count = 0;
        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            count += neighbors
                .iter()
                .filter(|&neighbor| {
                    if nearly_merge {
                        self.are_nearly_mergable(&card, neighbor, filter)
                    } else {
                        self.can_merge(&card, neighbor, filter)
                    }
                })
                .count();
        });
        (count / 2) as f64
    }

    fn can_merge(&self, left: &Card, right: &Card, filter: Option<&dyn ValueFilter>) -> bool {
        *left > 0
            && *right > 0
            && *left < Card::MAX
            && *right < Card::MAX
            && filter.is_none_or(|filter| filter.filter_values(&[*left, *right]))
            && (*left + *right == 3 || (*left > 2 && *left == *right))
    }

    fn are_nearly_mergable(
        &self,
        left: &Card,
        right: &Card,
        filter: Option<&dyn ValueFilter>,
    ) -> bool {
        // 1 with 3
        // 2 with 3
        // anything else with 2x itself or 0.5x itself
        *left > 0
            && *right > 0
            && *left < Card::MAX
            && *right < Card::MAX
            && filter.is_none_or(|filter| filter.filter_values(&[*left, *right]))
            && ((*left < 3 && *right == 3 || *left == 3 && *right < 3)
                || (*left >= 3 && *right >= 3 && (*left == *right * 2 || *left * 2 == *right)))
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos::{Merges, NearlyMerges};

    use super::super::super::wrappers::ValueFilterWrapper;
    use super::super::test_utils::generate_game_state;

    #[test]
    #[rustfmt::skip]
    fn test_merges() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0,
            Merges.merges(&game_state, None),
            "max score when everything is mergeable",
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(0.0, Merges.merges(&game_state, None), "no merges gives a score of 0");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1.0, Merges.merges(&game_state, None), "1 merge gives a score of 1");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(4.0, Merges.merges(&game_state, None), "4 pairs of merges in a 2x2");

        let game_state = generate_game_state([
            1, 2, 2, 0,
            0, 2, 2, 1,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ]);
        assert_eq!(2.0, Merges.merges(&game_state, None), "1's and 2's only merge with their counterpart");

        let game_state = generate_game_state([
            3, 3, 2, 1,  // 2
            0, 3, 6, 6,  // 1
            2, 1, 0, 12, // 1
            1, 2, 1, 2,  // 3  <-- this one was surprising
         // 1  2  0  0
        ]);
        assert_eq!(
            (2 + 1 + 1 + 3 + 1 + 2 + 0 + 0) as f64,
            Merges.merges(&game_state, None),
            "a big messy example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_filtered_merges() {
        let filter = ValueFilterWrapper {
            wrapped: Merges,
            min_value_to_keep: 1,
            max_value_to_keep: 6,
        };

        let game_state = generate_game_state([12; 16]);
        assert_eq!(
            0.0,
            Merges.merges(&game_state, Some(&filter)),
            "score of 0 when nothing matches the filter",
        );

        let game_state = generate_game_state([6; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0,
            Merges.merges(&game_state, Some(&filter)),
            "max score when everything matches the filter",
        );

        let game_state = generate_game_state([
            12, 12, 0, 0,
            12, 12, 0, 0,
             0,  0, 6, 6,
             0,  0, 6, 6,
        ]);
        assert_eq!(4.0, Merges.merges(&game_state, Some(&filter)), "12s are ignored, 6s aren't");

        let filter = ValueFilterWrapper {
            wrapped: Merges,
            min_value_to_keep: 1,
            max_value_to_keep: 1,
        };
        let game_state = generate_game_state([
            3, 3, 6, 6,
            0, 0, 0, 0,
            1, 2, 3, 6, // 1/2 has a merge and 1 matches the filter
            0, 0, 0, 0,
        ]);
        assert_eq!(
            1.0, Merges.merges(&game_state, Some(&filter)),
            "1s can merge with 2s if either is matched by the filter"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_nearly_merges() {
        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(0.0, NearlyMerges.nearly_merges(&game_state, None), "0 when everything is {value}");
        }

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(0.0, NearlyMerges.nearly_merges(&game_state, None), "1s and 2s aren't nearly mergeable");

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(4.0, NearlyMerges.nearly_merges(&game_state, None), "1s and 2s merge with 3s");

        let game_state = generate_game_state([
             3, 6, 3, 12, // 2
             6, 0, 3,  0, // 0
            12, 3, 0, 12, // 0
             0, 0, 0, 24, // 0
        //   2  0  0   1
        ]);
        assert_eq!(
            5.0, NearlyMerges.nearly_merges(&game_state, None),
            "Cards merge with cards twice their value"
        );

        let game_state = generate_game_state([
            1, 6, 3,  2, // 2
            6, 0, 3,  0, // 0
            2, 3, 0, 12, // 1
            0, 0, 0, 24, // 0
        //  0  0  0   1
        ]);
        assert_eq!(4.0, NearlyMerges.nearly_merges(&game_state, None), "A mix of everything");
    }

    #[test]
    #[rustfmt::skip]
    fn test_filtered_nearly_merges() {
        let filter = ValueFilterWrapper {
            wrapped: Merges,
            min_value_to_keep: 1,
            max_value_to_keep: 6,
        };

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(
            0.0, NearlyMerges.nearly_merges(&game_state, Some(&filter)),
            "Even though they match the filter, 1s and 2s aren't nearly mergeable"
        );

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(
            4.0, NearlyMerges.nearly_merges(&game_state, Some(&filter)),
            "1s and 2s match the filter and merge with 3s"
        );

        let game_state = generate_game_state([
            24, 12, 6, 3,
             0,  0, 6, 0,
             0,  0, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            2.0, NearlyMerges.nearly_merges(&game_state, Some(&filter)),
            "6 passes the filter and is nearly mergeable with the values on either side of it"
        );

    }
}
