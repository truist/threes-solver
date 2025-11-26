use threes_simulator::game_state::{Card, GameState};

use super::super::neighbors::iterate_with_neighbors;

// cards that can merge with each other
pub(crate) fn merges(game_state: &GameState) -> u8 {
    let mut count = 0;
    iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
        let new_count = neighbors
            .iter()
            .filter(|&neighbor| can_merge(&card, neighbor))
            .count() as u8;
        count += new_count;
    });
    count / 2
}
fn can_merge(left: &Card, right: &Card) -> bool {
    *left > 0
        && *right > 0
        && *left < Card::MAX
        && *right < Card::MAX
        && (*left + *right == 3 || (*left > 2 && *left == *right))
}

// cards that are one off from merging with each other (e.g. 3 and 6)
pub(crate) fn nearly_merges(game_state: &GameState) -> u8 {
    let mut count = 0;
    iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
        let new_count = neighbors
            .iter()
            .filter(|&neighbor| are_nearly_mergable(&card, neighbor))
            .map(|_| 1 as u8)
            .sum::<u8>();
        count += new_count;
    });
    count / 2
}
fn are_nearly_mergable(left: &Card, right: &Card) -> bool {
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

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::super::test_utils::generate_game_state;

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_merges() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            // 3 pairs across each row
            // 4 rows
            // then repeat that for columns
            3 * 4 * 2,
            merges(&game_state),
            "max score when everything is mergeable",
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(0, merges(&game_state), "no merges gives a score of 0");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1, merges(&game_state), "1 merge gives a score of 1");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(4, merges(&game_state), "4 pairs of merges in a 2x2");

        let game_state = generate_game_state([
            1, 2, 2, 0,
            0, 2, 2, 1,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ]);
        assert_eq!(2, merges(&game_state), "1's and 2's only merge with their counterpart");

        let game_state = generate_game_state([
            3, 3, 2, 1,  // 2
            0, 3, 6, 6,  // 1
            2, 1, 0, 12, // 1
            1, 2, 1, 2,  // 3  <-- this one was surprising
         // 1  2  0  0
        ]);
        assert_eq!(
            2 + 1 + 1 + 3 + 1 + 2 + 0 + 0,
            merges(&game_state),
            "a big messy example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_nearly_merges() {
        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(0, nearly_merges(&game_state), "0 when everything is {value}");
        }

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(0, nearly_merges(&game_state), "1s and 2s aren't nearly mergeable");

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(4, nearly_merges(&game_state), "1s and 2s merge with 3s");

        let game_state = generate_game_state([
             3, 6, 3, 12, // 2
             6, 0, 3,  0, // 0
            12, 3, 0, 12, // 0
             0, 0, 0, 24, // 0
        //   2  0  0   1
        ]);
        assert_eq!(5, nearly_merges(&game_state), "Cards merge with cards twice their value");

        let game_state = generate_game_state([
            1, 6, 3,  2, // 2
            6, 0, 3,  0, // 0
            2, 3, 0, 12, // 1
            0, 0, 0, 24, // 0
        //  0  0  0   1
        ]);
        assert_eq!(4, nearly_merges(&game_state), "A mix of everything");
    }
}
