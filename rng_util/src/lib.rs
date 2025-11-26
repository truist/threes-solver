pub use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};
pub use rand::{Rng, RngCore, SeedableRng};

pub type RngType = rand_xoshiro::Xoshiro256PlusPlus;

pub fn rng_from_seed(seed: u64) -> RngType {
    RngType::seed_from_u64(seed)
}

pub fn seed_from_entropy() -> u64 {
    rand::rng().random()
}

pub fn initialize_rng(seed: Option<u64>) -> (RngType, u64) {
    let seed = if let Some(val) = seed {
        println!("Using user-provided seed: {val}");
        val
    } else {
        let seed = seed_from_entropy();
        println!("Generated random seed: {seed}");
        seed
    };

    (rng_from_seed(seed), seed)
}

pub fn derive_worker_rng(master: &RngType, worker_index: usize) -> RngType {
    let mut worker = master.clone();
    for _ in 0..worker_index {
        worker.jump();
    }
    worker
}

pub fn test_rng() -> RngType {
    rng_from_seed(0)
}
