use threes_simulator::board_state::BOARD_SIZE;
use threes_simulator::game_state::{Card, Direction};

pub(crate) const NUM_NEIGHBORS: usize = 4;

pub(crate) const NEIGHBOR_INDICES: [[usize; NUM_NEIGHBORS]; BOARD_SIZE * BOARD_SIZE] = {
    let mut neighbor_indices = [[usize::MAX; NUM_NEIGHBORS]; BOARD_SIZE * BOARD_SIZE];

    let mut i = 0;
    while i < neighbor_indices.len() {
        let row = i / BOARD_SIZE;
        let col = i % BOARD_SIZE;

        if col > 0 {
            neighbor_indices[i][Direction::Left as usize] = i - 1;
        }
        if row > 0 {
            neighbor_indices[i][Direction::Up as usize] = i - BOARD_SIZE;
        }
        if col < BOARD_SIZE - 1 {
            neighbor_indices[i][Direction::Right as usize] = i + 1;
        }
        if row < BOARD_SIZE - 1 {
            neighbor_indices[i][Direction::Down as usize] = i + BOARD_SIZE;
        }

        i += 1;
    }

    neighbor_indices
};

// loop unrolled for a slight performance improvement
pub(crate) fn iterate_with_neighbors<F>(grid: &[Card], mut f: F)
where
    F: FnMut(usize, Card, [Card; NUM_NEIGHBORS]),
{
    let len = grid.len();
    for i in 0..len {
        let idxs = &NEIGHBOR_INDICES[i];

        let mut n0 = Card::MAX;
        if idxs[0] != usize::MAX {
            n0 = grid[idxs[0]];
        }

        let mut n1 = Card::MAX;
        if idxs[1] != usize::MAX {
            n1 = grid[idxs[1]];
        }

        let mut n2 = Card::MAX;
        if idxs[2] != usize::MAX {
            n2 = grid[idxs[2]];
        }

        let mut n3 = Card::MAX;
        if idxs[3] != usize::MAX {
            n3 = grid[idxs[3]];
        }

        f(i, grid[i], [n0, n1, n2, n3]);
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_iterate_with_neighbors() {
        let grid = [
             1,  2,  3,  4,
             5,  6,  7,  8,
             9, 10, 11, 12,
            13, 14, 15, 16
        ];

        let mut seen = [false; 16];

        iterate_with_neighbors(&grid, |index, card, neighbors| {
            match index {
                0 => check(index, card, [Card::MAX, Card::MAX, 2, 5], neighbors),
                1 => check(index, card, [1, Card::MAX, 3, 6], neighbors),
                2 => check(index, card, [2, Card::MAX, 4, 7], neighbors),
                3 => check(index, card, [3, Card::MAX, Card::MAX, 8], neighbors),
                4 => check(index, card, [Card::MAX, 1, 6, 9], neighbors),
                5 => check(index, card, [5, 2, 7, 10], neighbors),
                6 => check(index, card, [6, 3, 8, 11], neighbors),
                7 => check(index, card, [7, 4, Card::MAX, 12], neighbors),
                8 => check(index, card, [Card::MAX, 5, 10, 13], neighbors),
                9 => check(index, card, [9, 6, 11, 14], neighbors),
                10 => check(index, card, [10, 7, 12, 15], neighbors),
                11 => check(index, card, [11, 8, Card::MAX, 16], neighbors),
                12 => check(index, card, [Card::MAX, 9, 14, Card::MAX], neighbors),
                13 => check(index, card, [13, 10, 15, Card::MAX], neighbors),
                14 => check(index, card, [14, 11, 16, Card::MAX], neighbors),
                15 => check(index, card, [15, 12, Card::MAX, Card::MAX], neighbors),
                unexpected => assert!(false, "Unexpected board index: {unexpected}"),
            }
            seen[index] = true;
        });

        for position in 0..15 {
            assert_eq!(
                true, seen[position],
                "We were called for board position {}",
                position
            );
        }
    }

    fn check(
        index: usize,
        actual_value: Card,
        expected_neighbors: [Card; NUM_NEIGHBORS],
        actual_neighbors: [Card; NUM_NEIGHBORS],
    ) {
        assert_eq!(
            index as u16 + 1,
            actual_value,
            "Card is as expected at board position {index}"
        );
        assert_eq!(
            expected_neighbors, actual_neighbors,
            "Neighbors are as expected at board position {index}"
        );
    }
}
