use threes_simulator::game_state::{Card, Direction, GameState};

use super::super::neighbors::iterate_with_neighbors;

// a smaller card "squeezed" between bigger cards and/or the wall
pub(crate) fn squeezes(game_state: &GameState) -> u8 {
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

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::super::test_utils::generate_game_state;

    use super::*;

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
}
