# threes_solver

A tool that attempts to find an optimal algorithm for playing [Threes!](https://en.wikipedia.org/wiki/Threes) by simulating millions of games (using `threes_simulator`) and searching for the best set of weights for a variety of algorithms that calculate a score for a board state.

I developed this more to teach myself rust then to hit any particular goal, but I still hit at least one significant milestone: with `--lookahead-depth 3`, and optimized weights, the tool can play until a `6144` tile is generated, in a small percentage of runs.


## Dependencies

You'll need `fontconfig`.

On Mac (via HomeBrew):

```sh
brew install fontconfig
```

On Linux (with `apt`):

```sh
sudo apt-get install libfontconfig1-dev
```


## Using the tool

```sh
> cargo run --release -- help
Usage: threes_solver [OPTIONS] [COMMAND]

Commands:
  optimize  Default subcommand to discover optimal weights
  simulate  Optional subcommand to run a single game, showing each step
  help      Print this message or the help of the given subcommand(s)

Options:
      --seed <SEED>
          Set the seed for the RNG (u64)
      --weights-file <WEIGHTS_FILE>
          Path to read or write the weights TOML file [default: weights.toml]
      --lookahead-depth <LOOKAHEAD_DEPTH>
          How far to look ahead [default: 2]
      --single-insertion-point
          Do NOT evaluate (and average) all possible next-card insertion points
      --profiling
          Profiling mode (single thread, fewer generations)
      --max-threads <MAX_THREADS>
          Max threads to use [default: 0]
  -h, --help
          Print help
```

Note that `simulate` has further options (which are optional):

```sh
> cargo run --release -- simulate --help
Optional subcommand to run a single game, showing each step

Usage: threes_solver simulate [OPTIONS]

Options:
      --batch     Simulate a batch of games and report the aggregate results
      --insights  Provide higher-level insights into the choices made
  -h, --help      Print help
```

### To generate weights and then run the tool with those weights:

`cargo run --release -- optimize`

(Depending on your computer, wait anywhere from a few hours to a few days. Tuning variables are in the source.)

`cargo run --release -- simulate`

But generally you'll get better results with `--lookahead-depth 3` (on both commands) - but it will be much slower.


## Performance

Here's an example run on my 9-year-old iMac:

```sh
> ./simulate.sh --batch
Generated random seed: 17661324996810780838

Built under macOS 13.7.8 Ventura on Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz with 4 cores.
Built with Rust 1.92.0 (stable channel).
Built from git hash d38512b568ee7154869aff7a8b7aef701e4e779e with local modifications and Cargo.lock hash d1d21b68765e3065ae4d397fe2803e4cdec6c8292a46b03023cac003bc400b1c.
Running under Darwin 13.7.8 (kernel 22.6.0) on Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz with 4 cores.

Using weights: [3.534897400433314, 0.6460175715366324, 1.6262235944396892, 0.5377563513909953, 1.3494109624246835, 0.8034798600289569, 1.1033473536892653, 1.6873102904422586, 0.4248659443371991, 0.5046742184082651, 0.8513794735961553]
Running batch of 5000 games, with lookahead 2, evaluating all insertion point(s) per shift, with 4 threads
It took 87s; 57 games/s

96: 1
192: 91
384: 806
768: 2540
1536: 1507
3072: 55
```

(Note that 5000 games implies something on the order of 3,500,000 moves, and something on the order of 50,000,000 board state evaluations, all in 87 seconds.)

A modern M4 Max processor runs about 7x faster.

`--lookahead-depth 3` runs about 5x slower.

I've never seen a `6144` except with `--lookahead-depth 3`, but I've also never done a full "optimize all the weights as much as possible" optimization run - it would probably take weeks to finish optimizing.


## License

[GPL v3.0](LICENSE-GPL3)


