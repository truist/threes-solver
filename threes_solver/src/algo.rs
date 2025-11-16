use strum_macros::{EnumCount, EnumIter};

use threes_simulator::board_state::BOARD_SIZE;
use threes_simulator::game_state::{Card, Direction, GameState};

const NUM_NEIGHBORS: usize = 4;

const NEIGHBOR_INDICES: [[usize; NUM_NEIGHBORS]; BOARD_SIZE * BOARD_SIZE] = {
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
fn iterate_with_neighbors<F>(grid: &[Card], mut f: F)
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

#[derive(Debug, EnumCount, EnumIter)]
pub enum Algos {
    Empties,
    Merges,
    NearlyMerges,
    Squeezes,
    HighWall,
    HighCorner,
    Monotones,
}

pub trait Algo {
    fn score(&self, game_state: &Option<GameState>, _last_move_dir: &Direction) -> i8;
}

impl Algo for Algos {
    fn score(&self, game_state: &Option<GameState>, _last_move_dir: &Direction) -> i8 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => empties(game_state) as i8,
                Algos::Merges => merges(game_state) as i8,
                Algos::NearlyMerges => nearly_merges(game_state) as i8,
                Algos::Squeezes => squeezes(game_state) as i8 * -1,
                Algos::HighWall => high_walls(game_state) as i8,
                Algos::HighCorner => high_corners(game_state) as i8,
                Algos::Monotones => monotones(game_state) as i8,
            }
        } else {
            0
        }
    }
}

// cells that are empty
fn empties(game_state: &GameState) -> u8 {
    game_state
        .get_grid()
        .iter()
        .map(|&card| if card > 0 { 0 } else { 1 })
        .sum::<u8>()
}

// cards that can merge with each other
fn merges(game_state: &GameState) -> u8 {
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
fn nearly_merges(game_state: &GameState) -> u8 {
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

// a smaller card "squeezed" between bigger cards and/or the wall
fn squeezes(game_state: &GameState) -> u8 {
    let mut count = 0;

    iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
        for pair in [
            (
                &neighbors[Direction::Left as usize],
                &neighbors[Direction::Right as usize],
            ),
            (
                &neighbors[Direction::Up as usize],
                &neighbors[Direction::Down as usize],
            ),
        ] {
            if is_pair_squeezing(&card, pair) {
                count += 1;
            }
        }
    });

    count
}
fn is_pair_squeezing(middle: &Card, pair: (&Card, &Card)) -> bool {
    // this 'cleverly' takes advantage of wall-side "neighbors" being Card::MAX
    *middle > 0 && *pair.0 > *middle && *pair.1 > *middle
}

// Higher values near a wall.
fn high_walls(game_state: &GameState) -> u8 {
    high_impl(game_state, false)
}

// Higher values in a corner.
fn high_corners(game_state: &GameState) -> u8 {
    high_impl(game_state, true)
}

// Specifically, find the top three *actual* highest values on the board,
// then give a point for each card with one of those values next to a wall or in a corner.
// A card in a corner will only be counted once.
// Give 1 point for the lowest of the three, 2 for the middle, and 3 for the highest.
#[rustfmt::skip]
fn high_impl(game_state: &GameState, corners_only: bool) -> u8 {
    let grid = game_state.get_grid();

    // https://americanliterature.com/childrens-stories/goldilocks-and-the-three-bears
    let (mut great_big, mut middle, mut wee) = (0, 0, 0);
    for card in grid.iter() {
        if *card > great_big {
            wee = middle;
            middle = great_big;
            great_big = *card;
        } else if *card < great_big && *card > middle {
            wee = middle;
            middle = *card;
        } else if *card < middle && *card > wee {
            wee = *card;
        }
    }

    let mut score = 0;
    iterate_with_neighbors(grid, |index, card, _neighbors| {
        let mut count_cell = false;
        if card > 0 {
            if corners_only {
                count_cell = index == 0                              // top left
                    || index == BOARD_SIZE - 1                       // top right
                    || index == BOARD_SIZE * BOARD_SIZE - BOARD_SIZE // bottom left
                    || index == BOARD_SIZE * BOARD_SIZE - 1;         // bottom right
            } else {
                count_cell = index % BOARD_SIZE == 0                  // left wall
                    || index < BOARD_SIZE                             // top wall
                    || (index + 1) % BOARD_SIZE == 0                  // right wall
                    || index >= BOARD_SIZE * BOARD_SIZE - BOARD_SIZE; // bottom wall
            }
        }

        if count_cell {
            if card == great_big {
                score += 3;
            } else if card == middle {
                score += 2;
            } else if card == wee {
                score += 1;
            }
        }
    });

    score as u8
}

// Add points when left->right is monotonically increasing or decreasing, per row.
// Ditto for up->down, per col.
// Subtract points when they aren't.
// It's OK for neighboring rows (or cols) to run in opposite directions.
// 1s and 2s are treated as distinct values.
fn monotones(game_state: &GameState) -> u8 {
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

#[derive(Debug)]
pub struct WeightedAlgo {
    pub algo: Algos,
    pub weight: f64,
}

impl WeightedAlgo {
    pub fn score(&self, game_state: &Option<GameState>, last_move_dir: &Direction) -> f64 {
        let score = self.algo.score(game_state, last_move_dir);
        assert!(
            score < i8::MAX / 2,
            "{:?} doesn't take the i8 score limit into account (got score of {score})",
            self.algo
        );
        score as f64 * self.weight
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;

    use rng_util::test_rng;
    use threes_simulator::board_state::BoardState;
    use threes_simulator::draw_pile::DrawPile;
    use threes_simulator::game_state::Grid;

    fn generate_game_state(grid: Grid) -> GameState {
        let mut rng = test_rng();
        let mut draw_pile = DrawPile::initialize(&mut rng);
        let next = draw_pile.draw(&mut rng);

        let board_state = BoardState::initialize_test_state(grid, *grid.iter().max().unwrap());

        GameState::initialize_test_state(board_state, draw_pile, next)
    }

    #[test]
    fn test_score() {
        assert_eq!(
            0,
            Algos::Empties.score(&None, &Direction::Left),
            "all 'None' states get a 0 score"
        );

        let mut grid = [0; 16];
        grid[1] = 1;
        let game_state = generate_game_state(grid);

        assert!(
            Algos::Empties.score(&Some(game_state), &Direction::Left) > 0,
            "with a valid GameState, the score is greater than 0"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_empties() {
        let game_state = generate_game_state([
            3, 3, 3, 0,
            0, 3, 3, 3,
            0, 3, 3, 0,
            0, 0, 0, 0,
        ]);

        assert_eq!(8, empties(&game_state), "empty cells are counted correctly");
    }

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

    #[test]
    #[rustfmt::skip]
    fn test_squeezes() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            0,
            squeezes(&game_state),
            "no squeezes when everything is mergeable"
        );

        let mut grid = [0; 16];
        grid[5] = 3;
        let game_state = generate_game_state(grid);
        assert_eq!(
            0,
            squeezes(&game_state),
            "no squeezes when just one cell filled"
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(
            0,
            squeezes(&game_state),
            "no squeezes when cards don't have neighbors"
        );

        let game_state = generate_game_state([
            6, 3, 3, 6,
            3, 0, 0, 3,
            3, 0, 0, 3,
            6, 3, 3, 6,
        ]);
        assert_eq!(
            0,
            squeezes(&game_state),
            "no squeezes when squeezed cards are mergeable"
        );

        let game_state = generate_game_state([
            24, 12, 6, 0,
            12,  6, 3, 0,
             6,  3, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            0,
            squeezes(&game_state),
            "no squeezes when bigger neighbors aren't on opposite sides"
        );

        let game_state = generate_game_state([
            6, 3, 6, 0,
            3, 0, 0, 6,
            6, 0, 0, 3,
            0, 6, 3, 6,
        ]);
        assert_eq!(
            4,
            squeezes(&game_state),
            "obvious squeezes in both directions"
        );

        let game_state = generate_game_state([
            3, 6, 0, 3,
            0, 0, 0, 6,
            6, 0, 0, 0,
            3, 0, 6, 3,
        ]);
        assert_eq!(
            4,
            squeezes(&game_state),
            "you can be squeezed between a card and the wall, in all four directions"
        );

        let game_state = generate_game_state([
            24, 12, 6, 3,
             0,  0, 0, 0,
             0,  0, 0, 0,
             0,  0, 0, 0,
        ]);
        assert_eq!(
            1,
            squeezes(&game_state),
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
            1,
            squeezes(&game_state),
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
            2 + 2 + 1 + 0 + 0 + 0 + 1 + 2,
            squeezes(&game_state),
            "a big complex example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_high_walls() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(12 * 3, high_walls(&game_state), "36 when all cells have the same value");
        }

        let game_state = generate_game_state([
            3, 3, 3, 3,
            3, 0, 0, 3,
            3, 0, 0, 3,
            3, 3, 3, 3,
        ]);
        assert_eq!(12 * 3, high_walls(&game_state), "double check that all wall spots are counted");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(0, high_walls(&game_state), "0 when no values are near a wall");

        let game_state = generate_game_state([
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "3 when the highest high value is near a wall");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "still 3 when the highest value is in a corner");

        let game_state = generate_game_state([
            3,  3,  0, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(0, high_walls(&game_state), "0 when only non-highest values are near the walls");

        let game_state = generate_game_state([
            3,  3, 24, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(2, high_walls(&game_state), "2 when medium-high value is next to a wall, and other high values aren't");

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(4, high_walls(&game_state), "The top 3 *actual* high values are used, even if there's a gap");

        let game_state = generate_game_state([
            3, 96, 12, 0,
            3,  3, 24, 0,
            1,  2, 48, 0,
            0,  0,  0, 0,
        ]);
        assert_eq!(3, high_walls(&game_state), "A more complex example");
    }

    #[test]
    #[rustfmt::skip]
    fn test_high_corners() {
        for value in 1..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(4 * 3, high_corners(&game_state), "12 when all cells have the same value");
        }

        let game_state = generate_game_state([
            3, 0, 0, 3,
            0, 0, 0, 0,
            0, 0, 0, 0,
            3, 0, 0, 3,
        ]);
        assert_eq!(4 * 3, high_corners(&game_state), "double check that all corner spots are counted");

        let game_state = generate_game_state([
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 1,
            0, 0, 0, 0,
        ]);
        assert_eq!(0, high_corners(&game_state), "0 when no values are in a corner");

        let game_state = generate_game_state([
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(3, high_corners(&game_state), "3 when the highest high value is in a corner");

        let game_state = generate_game_state([
            3,  3,  0, 3,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0, 48,  0, 0,
        ]);
        assert_eq!(0, high_corners(&game_state), "0 when only non-highest values are in the corners");

        let game_state = generate_game_state([
            24, 3,  3, 0,
            3, 48, 24, 0,
            0,  0, 12, 0,
            0,  0,  0, 3,
        ]);
        assert_eq!(2, high_corners(&game_state), "2 when medium-high value is in a corner, and other high values aren't");

        let game_state = generate_game_state([
            96, 3, 12, 0,
            3,  3, 24, 0,
            0,  0, 12, 0,
            12, 0,  0, 0,
        ]);
        assert_eq!(4, high_corners(&game_state), "The top 3 *actual* high values are used, even if there's a gap");

        let game_state = generate_game_state([
            24, 96, 3, 96,
            3,  3, 24,  0,
            1,  2, 48,  0,
            48, 0,  0, 24,
        ]);
        assert_eq!(7, high_corners(&game_state), "A more complex example");
    }

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

/*
    TODO: more algos

    done:
        empty squares
        mergable cards next to each other
        off-by-one cards next to each other
        penalty for "trapped" numbers (lower between two higher)
        high(er) values on a wall (vs. in the middle)
        high values in a corner
        lower values (e.g. 1 & 2s) on the opposite wall/corner from higher values

    cross-cutting:
        boost scores (and penalties) when it's 1's and 2's vs. other values
        boost scores (and penalties) when it's high values?
        or both, and leave "mid" alone?
        have "early-game" vs. "late-game" algos

    needs context beyond the current board state:
        lookahead
        (so impl would be in solver.rs, not algo.rs)
        "expect" a 1 or 2, "soon", based on:
            time since last 1/2 (both ways)
            1/2 imbalance (both ways)
            the number of 1's (and 2's) in the last 12 cards
        most moveable directions
        most future move possibilities (down a given path)
        best best-case future
        penalize having few future move possibilities
        penalize worst worst-case future

    might be covered by lookahead or nuances of other algos:
        (so value might be low)
        modify existing algos to give points for "reachable" pairs:
            (adjust for distance?)
            mergeable
            off-by-one
        just one card of the biggest size
            just one card of each of the biggest sizes
        bigger groups of empty spaces
        more "areas" of empty spaces
        adjacent sequences of the same number
        adjacent sequences of single increments (including 2 -> 1 -> 3)
        bonus for multi-direction adjacency (gives more move options)
        higher values clustered together
        1s & 2s near a wall and shiftable away (i.e. to allow matches in)

    existing-algo modifications:
        (value might be minimal)
        (maybe implement as new algos, to see which version gets selected)
        only count "trapped" wall-cases when shifting is blocked toward the wall
        penalty for having "too many" non-mergable cards next to you
        some idea other than "top 3" for wall and corner credit

*/
