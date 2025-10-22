mod algo;
mod optimizer;
mod solver;

fn main() {
    let optimal_weights = optimizer::find_optimal_weights();
    println!("Optimal weights: {optimal_weights:?}");
}
