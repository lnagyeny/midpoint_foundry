use std::time::Instant;

use crate::algorithms::{
    baseline_circle::BaselineCircle, gap_fill_circle::GapFillCircle,
    midpoint_circle::MidpointCircle, midpoint_line::MidpointLine,
    parallel_midpoint_circle::ParallelMidpointCircle, sitaraman_fill_circle::SitaramanFillCircle,
    Algorithm,
};

// ── Benchmark Statistics ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct BenchmarkStats {
    pub min_us: f64, // microseconds
    pub max_us: f64,
    pub avg_us: f64,
    pub median_us: f64,
}

// ── Benchmarking ─────────────────────────────────────────────────────────────

pub fn run_benchmark(radius: i32) -> Vec<(String, BenchmarkStats)> {
    const ITERATIONS: usize = 100;
    let mut results = Vec::new();

    // Helper function to benchmark an algorithm with a constructor closure
    fn benchmark_algorithm<F>(
        name: &str,
        constructor: F,
        iterations: usize,
    ) -> (String, BenchmarkStats)
    where
        F: Fn() -> Box<dyn Algorithm>,
    {
        let mut algo = constructor();
        algo.compute(); // Warm up
        let times = time_algorithm(|| algo.compute(), iterations);
        (name.to_string(), calculate_stats(&times))
    }

    fn time_algorithm<F: FnMut()>(mut compute: F, iterations: usize) -> Vec<f64> {
        let mut times = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            compute();
            let elapsed_us = start.elapsed().as_secs_f64() * 1_000_000.0;
            times.push(elapsed_us);
        }
        times
    }

    // Benchmark all algorithms
    results.push(benchmark_algorithm(
        "Baseline Circle",
        || {
            let mut algo = Box::new(BaselineCircle::default());
            algo.radius = radius;
            algo
        },
        ITERATIONS,
    ));

    results.push(benchmark_algorithm(
        "Midpoint Circle",
        || {
            let mut algo = Box::new(MidpointCircle::default());
            algo.radius = radius;
            algo
        },
        ITERATIONS,
    ));

    results.push(benchmark_algorithm(
        "Gap-Fill Circle",
        || {
            let mut algo = Box::new(GapFillCircle::default());
            algo.radius = radius;
            algo
        },
        ITERATIONS,
    ));

    results.push(benchmark_algorithm(
        "Sitaraman Fill Circle",
        || {
            let mut algo = Box::new(SitaramanFillCircle::default());
            algo.radius = radius;
            algo
        },
        ITERATIONS,
    ));

    results.push(benchmark_algorithm(
        "Midpoint Line",
        || Box::new(MidpointLine::default()),
        ITERATIONS,
    ));

    results.push(benchmark_algorithm(
        "Parallel Midpoint Circle",
        || {
            let mut algo = Box::new(ParallelMidpointCircle::default());
            algo.radius = radius;
            algo
        },
        ITERATIONS,
    ));

    results
}

fn calculate_stats(times: &[f64]) -> BenchmarkStats {
    let mut sorted = times.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_us = sorted[0];
    let max_us = sorted[sorted.len() - 1];
    let avg_us = times.iter().sum::<f64>() / times.len() as f64;
    let median_us = sorted[sorted.len() / 2];

    BenchmarkStats {
        min_us,
        max_us,
        avg_us,
        median_us,
    }
}
