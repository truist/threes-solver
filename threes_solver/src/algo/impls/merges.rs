use threes_simulator::game_state::{Card, GameState};

use crate::algo::core::Algos;

use super::super::core::ValueBooster;
use super::super::neighbors::iterate_with_neighbors;

impl Algos {
    // cards that can merge with each other
    pub(crate) fn merges(&self, game_state: &GameState, booster: Option<&dyn ValueBooster>) -> f64 {
        self.merge_impl(false, game_state, booster)
    }

    // cards that are one off from merging with each other (e.g. 3 and 6)
    pub(crate) fn nearly_merges(
        &self,
        game_state: &GameState,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
        self.merge_impl(true, game_state, booster)
    }

    fn merge_impl(
        &self,
        nearly_merge: bool,
        game_state: &GameState,
        booster: Option<&dyn ValueBooster>,
    ) -> f64 {
        let mut score = 0.0;
        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            score += neighbors
                .iter()
                .filter(|&neighbor| {
                    if nearly_merge {
                        self.are_nearly_mergable(&card, neighbor)
                    } else {
                        self.can_merge(&card, neighbor)
                    }
                })
                .map(|&neighbor| {
                    if let Some(booster) = booster {
                        booster.boost_score_for(1.0, &[card, neighbor])
                    } else {
                        1.0
                    }
                })
                .sum::<f64>();
        });
        score / 2.0
    }

    fn can_merge(&self, left: &Card, right: &Card) -> bool {
        *left > 0
            && *right > 0
            && *left < Card::MAX
            && *right < Card::MAX
            && (*left + *right == 3 || (*left > 2 && *left == *right))
    }

    fn are_nearly_mergable(&self, left: &Card, right: &Card) -> bool {
        // 1 with 3
        // 2 with 3
        // anything else with 2x itself or 0.5x itself
        *left > 0
            && *right > 0
            && *left < Card::MAX
            && *right < Card::MAX
            && ((*left < 3 && *right == 3 || *left == 3 && *right < 3)
                || (*left >= 3 && *right >= 3 && (*left == *right * 2 || *left * 2 == *right)))
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use crate::algo::core::Algos::{Merges, NearlyMerges};

    use super::super::super::test_utils::generate_game_state;
    use super::super::super::wrappers::value_booster_wrapper::ValueBoosterWrapper;

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
    fn test_boosted_merges() {
        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: Merges,
            min_value_to_boost: 1,
            max_value_to_boost: 6,
            boost: test_boost,
        };

        let game_state = generate_game_state([12; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0,
            Merges.merges(&game_state, Some(&booster)),
            "default score when nothing matches the booster",
        );

        let game_state = generate_game_state([6; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3.0 * 4.0 * 2.0 * test_boost,
            Merges.merges(&game_state, Some(&booster)),
            "boosted score when everything matches the booster",
        );

        let game_state = generate_game_state([
            12, 12, 0, 0,
            12, 12, 0, 0,
             0,  0, 6, 6,
             0,  0, 6, 6,
        ]);
        assert_eq!(
            4.0 + 4.0 * test_boost, Merges.merges(&game_state, Some(&booster)),
            "12s are normal, 6s are boosted"
        );

        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: Merges,
            min_value_to_boost: 1,
            max_value_to_boost: 1,
            boost: test_boost,
        };
        let game_state = generate_game_state([
            3, 3, 6, 6, // 2 un-boosted merges
            0, 0, 0, 0, // 0
            1, 2, 3, 6, // 1/2 merges and is boosted by 1
            0, 0, 0, 0, // 0
        ]);
        assert_eq!(
            2.0 + 1.0 * test_boost, Merges.merges(&game_state, Some(&booster)),
            "1s can merge with 2s if either is matched by the booster"
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
    fn test_boosted_nearly_merges() {
        let test_boost = 2.5;
        let booster = ValueBoosterWrapper {
            wrapped: Merges,
            min_value_to_boost: 1,
            max_value_to_boost: 6,
            boost: test_boost,
        };

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(
            0.0, NearlyMerges.nearly_merges(&game_state, Some(&booster)),
            "Even though they match the booster, 1s and 2s aren't nearly mergeable"
        );

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(
            4.0 * test_boost, NearlyMerges.nearly_merges(&game_state, Some(&booster)),
            "1s and 2s match the booster and merge with 3s"
        );

        let game_state = generate_game_state([
            24, 12, 6, 3, // 24/12 not boosted; 12/6 boosted; 6/3 boosted
             0,  0, 6, 0,
             0,  0, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            1.0 + 1.0 * test_boost + 1.0 * test_boost, NearlyMerges.nearly_merges(&game_state, Some(&booster)),
            "6 is boosted and is nearly mergeable with the values on either side of it"
        );

    }

    #[test]
    #[rustfmt::skip]
    fn test_max_possible_score() {
        // 24
        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 3, 3, 3,
            3, 3, 3, 3,
            3, 3, 3, 3,
        ]);
        assert_eq!(3.0 * 4.0 * 2.0, Merges.merges(&game_state, None), "merges max score");

        // 24
        let game_state = generate_game_state([
            3, 6, 3, 6,
            6, 3, 6, 3,
            3, 6, 3, 6,
            6, 3, 6, 3,
        ]);
        assert_eq!(3.0 * 4.0 * 2.0, Merges.nearly_merges(&game_state, None), "nearly_merges max score");
    }
}
