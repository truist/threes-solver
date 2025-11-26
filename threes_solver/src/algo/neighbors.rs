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
