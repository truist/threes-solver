# threes-solver

This repo contains multiple Rust crates for simulating [Threes!](https://en.wikipedia.org/wiki/Threes), a game I first discovered (and loved!) on iOS many years ago, and then re-engaged with after I happened across the (old) [news that it took three years to beat it](https://www.reddit.com/r/Games/comments/6ids8k/over_three_years_later_someone_beat_threes/) and saw that there was a community of people who had developed algorithms to play it.

And I got annoyed that they all called their algorithms "A.I." even though they were just algorithms 😜.

And I was looking for an excuse to learn [Rust](https://rust-lang.org/), and building a game simulator / solver seemed like the perfect use-case for it: command-line driven code of high complexity, where native performance would be critical.

So I set about making a [Threes! simulator](threes_simulator/README.md), a [Threes! solver](threes_solver/README.md), and a few utility libraries.

And I learned a lot of Rust!

## Installation

I plan to publish these crates to [crates.io](https://crates.io/) but I haven't done so yet.
So for the moment, it only works by cloning this repo.

There are driver scripts in the two primary crates, but they are only written and tested on Mac.
In general they resolve down to `cargo run --release -- [<args>]`, and `-h` will work for `<args>`.

## Crates

* [`threes_solver`](threes_solver/README.md): the main crate that automatically tests and tunes weights for various solving algorithms to try to find an optimal play strategy for Threes!.
* [`threes_simulator`](threes_simulator/README.md): an implementation of the Threes! algorithm, as best as I understand it. You can actually play Threes! in a shell, with this crate!
* [`tee_output`](tee_output/README.md): a utility crate to automatically copy stdout and stderr to a log file. This seemed like something that should have already existed, but it didn't - I hope it will be generally useful.
* [`rng_util`](rng_util/README.md): a tiny crate to hide away Rng implementation details from the main crates.

## License

The crates in this repo have different licenses. `tee_output` and `rng_util` use the MIT license.
`threes_simulator` and `threes_solver` use the GPL v3.0.

## AI disclosure

See [AI_POLICY.md](AI_POLICY.md).

