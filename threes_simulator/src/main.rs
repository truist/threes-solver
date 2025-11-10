use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
    let (mut rng, _) = rng_util::initialize_rng(args.seed);

    threes_simulator::play(&mut rng);
}
