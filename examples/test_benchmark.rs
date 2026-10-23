use midpoint_foundry::benchmark::run_benchmark;

fn main() {
    let results = run_benchmark(20);
    println!("Benchmark results:");
    for (name, stats) in results {
        println!(
            "{}: avg={:.2}µs, min={:.2}µs, max={:.2}µs",
            name, stats.avg_us, stats.min_us, stats.max_us
        );
    }
}
