use strum_macros::{EnumCount, EnumIter};

use threes_simulator::game_state::{Card, Direction, GameState};

const NEIGHBOR_INDICES: [[usize; 4]; 16] = {
    let mut neighbor_indices = [[usize::MAX; 4]; 16];

    let mut i = 0;
    while i < 16 {
        let row = i / 4;
        let col = i % 4;

        if col > 0 {
            neighbor_indices[i][Direction::Left as usize] = i - 1;
        }
        if row > 0 {
            neighbor_indices[i][Direction::Up as usize] = i - 4;
        }
        if col < 3 {
            neighbor_indices[i][Direction::Right as usize] = i + 1;
        }
        if row < 3 {
            neighbor_indices[i][Direction::Down as usize] = i + 4;
        }

        i += 1;
    }

    neighbor_indices
};

// loop unrolled for a slight performance improvement
fn iterate_with_neighbors<F>(grid: &[Card], mut f: F)
where
    F: FnMut(usize, Card, [Card; 4]),
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
}

impl Algos {
    pub fn score(&self, game_state: &Option<GameState>, _last_move_dir: &Direction) -> i8 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state) as i8,
                Algos::Merges => self.merges(game_state) as i8,
                Algos::NearlyMerges => self.nearly_merges(game_state) as i8,
                Algos::Squeezes => self.squeezes(game_state) as i8 * -1,
            }
        } else {
            0
        }
    }

    // cells that are empty
    fn empties(&self, game_state: &GameState) -> u8 {
        game_state
            .get_grid()
            .iter()
            .map(|&card| if card > 0 { 0 } else { 1 })
            .sum::<u8>()
    }

    // cards that can merge with each other
    fn merges(&self, game_state: &GameState) -> u8 {
        let mut count = 0;
        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            let new_count = neighbors
                .iter()
                .filter(|&neighbor| self.can_merge(&card, neighbor))
                .map(|_| 1 as u8)
                .sum::<u8>();
            // println!("cell {_index} has {new_count} merges");
            count += new_count;
        });
        count / 2
    }
    fn can_merge(&self, left: &Card, right: &Card) -> bool {
        *left > 0
            && *right > 0
            && *left < Card::MAX
            && *right < Card::MAX
            && (*left + *right == 3 || (*left > 2 && *left == *right))
    }

    // cards that are one off from merging with each other (e.g. 3 and 6)
    fn nearly_merges(&self, game_state: &GameState) -> u8 {
        let mut count = 0;
        iterate_with_neighbors(game_state.get_grid(), |_index, card, neighbors| {
            let new_count = neighbors
                .iter()
                .filter(|&neighbor| self.are_nearly_mergable(&card, neighbor))
                .map(|_| 1 as u8)
                .sum::<u8>();
            // println!("cell {_index} has {new_count} merges");
            count += new_count;
        });
        count / 2
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

    // a smaller card "squeezed" between bigger cards and/or the wall
    fn squeezes(&self, game_state: &GameState) -> u8 {
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
                if self.is_pair_squeezing(&card, pair) {
                    count += 1;
                }
            }
        });

        count
    }

    fn is_pair_squeezing(&self, middle: &Card, pair: (&Card, &Card)) -> bool {
        // this 'cleverly' takes advantage of wall-side "neighbors" being Card::MAX
        *middle > 0 && *pair.0 > *middle && *pair.1 > *middle
    }
}

#[derive(Debug)]
pub struct WeightedAlgo {
    pub algo: Algos,
    pub weight: f64,
}

impl WeightedAlgo {
    pub fn score(&self, game_state: &Option<GameState>, last_move_dir: &Direction) -> f64 {
        self.algo.score(game_state, last_move_dir) as f64 * self.weight
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

        assert_eq!(8, Algos::Empties.empties(&game_state), "empty cells are counted correctly");
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
            Algos::Merges.merges(&game_state),
            "max score when everything is mergeable",
        );

        let game_state = generate_game_state([
            3, 0, 3, 0,
            0, 3, 0, 3,
            3, 0, 3, 0,
            0, 3, 0, 3,
        ]);
        assert_eq!(0, Algos::Merges.merges(&game_state), "no merges gives a score of 0");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(1, Algos::Merges.merges(&game_state), "1 merge gives a score of 1");

        let game_state = generate_game_state([
            3, 3, 0, 0,
            3, 3, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert_eq!(4, Algos::Merges.merges(&game_state), "4 pairs of merges in a 2x2");

        let game_state = generate_game_state([
            1, 2, 2, 0,
            0, 2, 2, 1,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ]);
        assert_eq!(2, Algos::Merges.merges(&game_state), "1's and 2's only merge with their counterpart");

        let game_state = generate_game_state([
            3, 3, 2, 1,  // 2
            0, 3, 6, 6,  // 1
            2, 1, 0, 12, // 1
            1, 2, 1, 2,  // 3  <-- this one was surprising
         // 1  2  0  0
        ]);
        assert_eq!(
            2 + 1 + 1 + 3 + 1 + 2 + 0 + 0,
            Algos::Merges.merges(&game_state),
            "a big messy example"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_nearly_merges() {
        for value in 0..=3 {
            let game_state = generate_game_state([value; 16]);
            assert_eq!(0, Algos::Merges.nearly_merges(&game_state), "0 when everything is {value}");
        }

        let game_state = generate_game_state([
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ]);
        assert_eq!(0, Algos::Merges.nearly_merges(&game_state), "1s and 2s aren't nearly mergeable");

        let game_state = generate_game_state([
            1, 2, 3, 0, // 1
            2, 1, 3, 0, // 1
            3, 3, 0, 0, // 0
            0, 0, 0, 0, // 0
        //  1  1  0  0
        ]);
        assert_eq!(4, Algos::Merges.nearly_merges(&game_state), "1s and 2s merge with 3s");

        let game_state = generate_game_state([
             3, 6, 3, 12, // 2
             6, 0, 3,  0, // 0
            12, 3, 0, 12, // 0
             0, 0, 0, 24, // 0
        //   2  0  0   1
        ]);
        assert_eq!(5, Algos::Merges.nearly_merges(&game_state), "Cards merge with cards twice their value");

        let game_state = generate_game_state([
            1, 6, 3,  2, // 2
            6, 0, 3,  0, // 0
            2, 3, 0, 12, // 1
            0, 0, 0, 24, // 0
        //  0  0  0   1
        ]);
        assert_eq!(4, Algos::Merges.nearly_merges(&game_state), "A mix of everything");
    }

    #[test]
    #[rustfmt::skip]
    fn test_squeezes() {
        let game_state = generate_game_state([3; 16]);
        assert_eq!(
            0,
            Algos::Squeezes.squeezes(&game_state),
            "no squeezes when everything is mergeable"
        );

        let mut grid = [0; 16];
        grid[5] = 3;
        let game_state = generate_game_state(grid);
        assert_eq!(
            0,
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
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
            Algos::Squeezes.squeezes(&game_state),
            "a big complex example"
        );
    }
}

/*
    TODO: more algos

    basics:
+       empty squares

+       mergable cards next to each other
            ditto, but for pairs that are "reachable", with adjustment for distance

+       off-by-one cards next to each other
            with adjustment for distance? (requires "reachability")

+       penalty for "trapped" numbers
+           lower between two higher
                but we should only count wall-cases when shifting is blocked toward the wall!
            for having "too many" non-mergable cards next to you?
                especially for 1's and 2's?

    cross-cutting:
        boost scores (and penalties) when it's 1's and 2's vs. other values
        or boost scores (and penalties) when it's high values?
        or both, and leave "mid" alone?

    advanced:
        just one card of the biggest size
            just one card of each of the biggest sizes

        bigger groups of empty spaces
        more "areas" of empty spaces

        high(er) values on a wall (vs. in the middle)
        high values in a corner (which is really just 2 walls)
        higher values clustered together
            maybe with extra bonus for being on a wall
        lower values (e.g. 1 & 2s) on the opposite wall/corner from higher values
        1s & 2s near a wall and shiftable away (i.e. to allow matches in)
            maybe covered by lookahead?

        adjacent sequences of the same number
            (or penalize for separation... that seems equivalent)
            with some notion of "distance" and "reachability"?
        adjacent sequences of single increments (including 2 -> 1 -> 3)
            (ditto prior idea)
        bonus for multi-direction adjacency (gives more move options)
            maybe this comes out automatically with look-ahead?

        keep track of the number of 1's (and 2's) in the last 12 cards
            and use that to adjust expectations of future cards

    scores that need context beyond the current board state (in solver.rs):
        most moveable directions
        most future move possibilities (down a given path)
        best best-case future
        penalize having few future move possibilities
        penalize worst worst-case future
        "expect" a 1 or 2, "soon", based on:
            time since last 1/2 (both ways)
            1/2 imbalance (both ways)
*/
