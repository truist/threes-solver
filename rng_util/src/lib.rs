pub use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};
pub use rand::{Rng, RngCore, SeedableRng};

use std::num::ParseIntError;

pub type RngType = rand_xoshiro::Xoshiro256PlusPlus;

pub fn rng_from_seed(seed: u64) -> RngType {
    RngType::seed_from_u64(seed)
}

pub fn seed_from_entropy() -> u64 {
    rand::rng().random()
}

pub fn parse_seed(s: &str) -> Result<u64, ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        u64::from_str_radix(s, 16).or_else(|_| s.parse::<u64>())
    }
}

// this will eventually have to be smarter, if/when the CLIs want more args than just '--seed'
pub fn parse_args() -> u64 {
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

    if let Some(s) = seed {
        let parsed = parse_seed(s.as_str()).unwrap();
        println!("Using user-provided seed: 0x{parsed:016x}");
        parsed
    } else {
        let seed = seed_from_entropy();
        println!("Generated random seed: 0x{seed:016x}");
        seed
    }
}

pub fn test_rng() -> RngType {
    rng_from_seed(0)
}

// Trait alias for function signatures
pub trait AnyRng: RngCore {}
impl<T: RngCore + ?Sized> AnyRng for T {}
