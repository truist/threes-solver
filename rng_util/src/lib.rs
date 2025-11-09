pub use rand::seq::{IteratorRandom, SliceRandom};
pub use rand::{Rng, RngCore, SeedableRng};

use std::num::ParseIntError;

type RngType = rand_xoshiro::Xoshiro256PlusPlus;

pub fn rng_from_seed(seed: u64) -> RngType {
    RngType::seed_from_u64(seed)
}

pub fn rng_from_entropy() -> RngType {
    let seed: u64 = rand::rng().random();
    println!("Generated random seed: 0x{seed:016x}");

    rng_from_seed(seed)
}

pub fn rng_from_user(seed_str: &str) -> RngType {
    let seed = parse_seed(&seed_str).expect("invalid seed format");
    println!("Using user-provided seed: 0x{seed:016x}");

    rng_from_seed(seed)
}

pub fn parse_seed(s: &str) -> Result<u64, ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        u64::from_str_radix(s, 16).or_else(|_| s.parse::<u64>())
    }
}

pub fn test_rng() -> RngType {
    rng_from_seed(0)
}

// Trait alias for function signatures
pub trait AnyRng: RngCore {}
impl<T: RngCore + ?Sized> AnyRng for T {}
