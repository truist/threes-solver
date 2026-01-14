use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sysinfo::System;

use rng_util::RngType;
use threes_simulator::game_state::Card;
use threes_simulator::game_state::GameState;

use algo::WeightedAlgo;
use optimizer::Optimizer;

mod algo;
mod optimizer;
use tee_output;
mod solver;

const DEFAULT_WEIGHTS_FILE_NAME: &str = "weights.toml";
const OPTIMIZE_LOG_FILE_NAME: &str = "run_logs/optimize.log";
const SIMULATE_BATCH_LOG_FILE_NAME: &str = "run_logs/simulate_batch.log";

#[derive(Parser)]
struct Args {
    /// Set the seed for the RNG (u64)
    #[arg(long)]
    seed: Option<u64>,

    /// Path to read or write the weights TOML file
    #[arg(long, default_value = DEFAULT_WEIGHTS_FILE_NAME)]
    weights_file: PathBuf,

    /// How far to look ahead
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=5))]
    lookahead_depth: u8,

    /// Do NOT evaluate (and average) all possible next-card insertion points
    #[arg(long)]
    single_insertion_point: bool,

    /// Profiling mode (single thread, fewer generations)
    #[arg(long)]
    profiling: bool,

    /// Max threads to use
    #[arg(long, default_value_t = 0)]
    max_threads: usize,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Default subcommand to discover optimal weights
    Optimize {
        /// Loosen the tolerances and stop earlier
        #[arg(long)]
        rough: bool,
    },

    /// Optional subcommand to run a single game, showing each step
    Simulate {
        /// Simulate a batch of games and report the aggregate results
        #[arg(long)]
        batch: bool,

        /// Provide higher-level insights into the choices made
        #[arg(long)]
        insights: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct WeightsConfig {
    pub weights: Vec<f64>,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Simulate { batch, insights }) => simulate(
            args.seed,
            args.weights_file,
            args.lookahead_depth as usize,
            !args.single_insertion_point,
            batch,
            insights,
            args.max_threads,
        ),

        Some(Commands::Optimize { rough }) => optimize(
            args.seed,
            args.max_threads,
            args.weights_file,
            args.lookahead_depth as usize,
            !args.single_insertion_point,
            args.profiling,
            rough,
        ),

        None => optimize(
            args.seed,
            0,
            args.weights_file,
            args.lookahead_depth as usize,
            !args.single_insertion_point,
            args.profiling,
            false,
        ),
    }
}

fn print_context() {
    println!();
    println!(
        "Built under {} on {} with {} cores.",
        option_env!("VERGEN_SYSINFO_OS_VERSION").unwrap_or("unknown"),
        option_env!("VERGEN_SYSINFO_CPU_BRAND").unwrap_or("unknown"),
        option_env!("VERGEN_SYSINFO_CPU_CORE_COUNT").unwrap_or("unknown"),
    );

    println!(
        "Built with Rust {} ({} channel).",
        option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown"),
        option_env!("VERGEN_RUSTC_CHANNEL").unwrap_or("unknown"),
    );

    let dirty = if let Some(dirty) = option_env!("VERGEN_GIT_DIRTY") {
        if dirty == "true" {
            ""
        } else {
            " no"
        }
    } else {
        " unknown"
    };
    println!(
        "Built from git hash {} with{} local modifications and Cargo.lock hash {}.",
        option_env!("VERGEN_GIT_SHA").unwrap_or("unknown"),
        dirty,
        option_env!("CARGO_LOCK_SHA256").unwrap_or("unknown"),
    );

    let sys = System::new_all();
    let brand = sys.cpus().first().map(|c| c.brand()).unwrap_or("unknown");
    println!(
        "Running under {} {} (kernel {}) on {} with {} cores.",
        System::name().unwrap_or("unknown".to_string()),
        System::os_version().unwrap_or("unknown".to_string()),
        System::kernel_version().unwrap_or("unknown".to_string()),
        brand,
        num_cpus::get_physical()
    );
    println!();
}

fn simulate(
    seed: Option<u64>,
    weights_file: PathBuf,
    lookahead_depth: usize,
    all_insertion_points: bool,
    batch: bool,
    insights: bool,
    max_threads: usize,
) {
    let _tee_guard = if batch {
        Some(init_logging(SIMULATE_BATCH_LOG_FILE_NAME))
    } else {
        None
    };

    let (mut rng, seed) = rng_util::initialize_rng(seed);
    print_context();

    let algos = crate::algo::build_all_algos();

    let weights_to_use = if weights_file.as_path().exists() {
        let toml_str = fs::read_to_string(weights_file).unwrap();
        let config: WeightsConfig = toml::from_str(&toml_str).unwrap();
        config.weights
    } else {
        eprintln!("Weights file ({weights_file:?}) doesn't exist; using default weights.");

        algos.iter().map(|_| 1.0).collect()
    };
    println!("Using weights: {weights_to_use:?}");

    let expected = algos.len();
    let actual = weights_to_use.len();
    if actual != expected {
        panic!("Incorrect number of weights supplied; expected {expected} but got {actual}");
    }

    let weighted_algos: Vec<WeightedAlgo> = algos
        .into_iter()
        .zip(weights_to_use.iter())
        .map(|(algo, &weight)| WeightedAlgo { algo, weight })
        .collect();

    if batch {
        run_batch(
            rng,
            seed,
            weighted_algos,
            lookahead_depth,
            all_insertion_points,
            max_threads,
        );
    } else {
        let verbose = if insights {
            solver::Verbose::Insights
        } else {
            solver::Verbose::Details
        };
        solver::play(
            GameState::initialize(&mut rng),
            &weighted_algos,
            lookahead_depth,
            all_insertion_points,
            &mut rng,
            verbose,
        );
    }
}

fn optimize(
    seed: Option<u64>,
    max_threads: usize,
    weights_file: PathBuf,
    lookahead_depth: usize,
    all_insertion_points: bool,
    profiling: bool,
    rough: bool,
) {
    let _tee_guard = init_logging(OPTIMIZE_LOG_FILE_NAME);

    let (mut rng, seed) = rng_util::initialize_rng(seed);
    print_context();

    let optimizer = Optimizer::new(
        &mut rng,
        seed,
        max_threads,
        lookahead_depth,
        all_insertion_points,
        profiling,
        rough,
    );
    let start = Instant::now();
    let optimal_weights = optimizer.find_optimal_weights();
    let duration = start.elapsed();
    println!("Optimizer ran for {duration:?}");

    let mut toml_weights = vec![];
    let algos: Vec<WeightedAlgo> = crate::algo::build_all_algos()
        .into_iter()
        .zip(optimal_weights.final_mean.iter())
        .map(|(algo, &weight)| {
            println!("{}: {}", algo, weight);

            toml_weights.push(weight);

            WeightedAlgo { algo, weight }
        })
        .collect();
    println!();

    let config = WeightsConfig {
        weights: toml_weights,
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    fs::write(weights_file, toml_str).unwrap();

    run_batch(
        rng,
        seed,
        algos,
        lookahead_depth,
        all_insertion_points,
        max_threads,
    );
}

fn run_batch(
    mut rng: RngType,
    seed: u64,
    weighted_algos: Vec<WeightedAlgo>,
    lookahead_depth: usize,
    all_insertion_points: bool,
    max_threads: usize,
) {
    let optimizer = Optimizer::new(
        &mut rng,
        seed,
        max_threads,
        lookahead_depth,
        all_insertion_points,
        false,
        false,
    );
    let workers = optimizer.make_worker_threads(Arc::new(weighted_algos), true);

    let game_count = optimizer::GAMES_PER_TEST;
    let insertion_point_desc = if all_insertion_points { "all" } else { "1" };
    println!(
        "Running batch of {} games, with lookahead {}, evaluating {} insertion point(s) per shift, with {} threads",
        game_count,
        lookahead_depth,
        insertion_point_desc,
        workers.len(),
    );

    let start = Instant::now();

    let mut high_cards: Vec<Card> = vec![];
    for handle in workers {
        high_cards.append(&mut handle.join().unwrap().1);
    }

    let elapsed = start.elapsed().as_secs().max(1);
    println!(
        "It took {elapsed}s; {} games/s\n",
        game_count as u64 / elapsed
    );

    let mut counts: BTreeMap<Card, usize> = BTreeMap::new();
    for high_card in high_cards {
        *counts.entry(high_card).or_insert(0) += 1;
    }
    for (card, count) in counts {
        println!("{card:?}: {count}");
    }
}

fn init_logging(file_name: &str) -> tee_output::TeeGuard {
    match tee_output::init_log_file(file_name) {
        Ok(guard) => guard,
        Err(e) => panic!("Error: Could not open log file {}: {}", file_name, e),
    }
}
