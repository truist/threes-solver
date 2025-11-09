fn main() {
    let mut rng = rng_util::rng_from_seed(rng_util::parse_args());
    threes_simulator::play(&mut rng);
}
