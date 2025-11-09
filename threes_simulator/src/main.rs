use rng_util::RngType;

fn parse_args() -> RngType {
    let mut args = std::env::args().skip(1);

    let mut seed: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                if let Some(val) = args.next() {
                    seed = Some(val);
                } else {
                    eprintln!("error: --seed requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("unknown argument: {arg}");
                eprintln!("args: [--seed <hex string>]");
                std::process::exit(1);
            }
        }
    }

    let (rng, _) = rng_util::initialize_rng(seed);
    rng
}

fn main() {
    threes_simulator::play(&mut parse_args());
}
