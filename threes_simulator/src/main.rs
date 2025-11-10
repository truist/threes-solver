use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, value_parser = clap::value_parser!(u64))]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
    let (mut rng, _) = rng_util::initialize_rng(args.seed);

    threes_simulator::play(&mut rng);
}
