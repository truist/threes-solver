pub use core::{build_all_algos, Algo};
pub use weighted::WeightedAlgo;

#[cfg(test)]
pub(crate) use core::Algos;

mod core;
mod impls;
mod neighbors;
mod weighted;
mod wrappers;

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
        have "early-game" vs. "late-game" algos
        boost scores (and penalties) when it's 1's and 2's vs. other values

    new ideas:
        penalize high values that aren't near a wall
        penalize few empty spots

    cross-cutting:
        boost scores (and penalties) when it's high values?
        or both, and leave "mid" alone?
        scale algos based on the number of empties
            i.e. some algos really matter when there are only a few

    needs context beyond the current board state:
        lookahead
        (so impl would be in solver.rs, not algo.rs)
        "expect" a 1 or 2, "soon", based on:
            time since last 1/2 (both ways)
            1/2 imbalance (both ways)
            the number of 1's (and 2's) in the last 12 cards
            or really maybe just keep a running idea of the probabilities of the possible next cards
                and use that in the lookahead
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
        bonus cards are often a good time to move in the "opposite" direction
        have a "primary" direction (toward high cards in a corner) and bias moves toward that direction

    existing-algo modifications:
        (value might be minimal)
        (maybe implement as new algos, to see which version gets selected)
        only count "trapped" wall-cases when shifting is blocked toward the wall
        penalty for having "too many" non-mergable cards next to you
        some idea other than "top 3" for wall and corner credit

*/
