//! Small, dependency-free timing harness. Run with `cargo run --release`.
use rust_dynamic_dispatch_overhead::{Transform, static_dispatch, vtable_dispatch};
use std::{env, hint::black_box, process, time::Instant};

const DEFAULT_ITERATIONS: u64 = 1_000_000;
const DEFAULT_SAMPLES: usize = 100;

fn measure(iterations: u64, run: impl FnOnce() -> u64) -> (f64, u64) {
    let start = Instant::now();
    let result = black_box(run());
    let elapsed = start.elapsed();
    (elapsed.as_secs_f64() * 1e9 / iterations as f64, result)
}

fn median(samples: &mut [f64]) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    let middle = samples.len() / 2;
    if samples.len().is_multiple_of(2) {
        (samples[middle - 1] + samples[middle]) / 2.0
    } else {
        samples[middle]
    }
}

fn report(label: &str, samples: &mut [f64], baseline: Option<f64>) -> f64 {
    let median = median(samples);
    let difference = baseline.map_or(0.0, |baseline| (median / baseline - 1.0) * 100.0);
    println!(
        "{label:<8} {median:>10.3} {min:>10.3} {max:>10.3} {difference:>+10.1}%",
        min = samples[0],
        max = samples[samples.len() - 1],
    );
    median
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 2 || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Usage: cargo run --release -- [iterations_per_sample] [samples]");
        println!("Defaults: {DEFAULT_ITERATIONS} iterations, {DEFAULT_SAMPLES} samples.");
        if args.len() > 2 {
            process::exit(2);
        }
        return;
    }
    let iterations = args
        .first()
        .map_or(Ok(DEFAULT_ITERATIONS), |s| s.parse::<u64>());
    let count = args
        .get(1)
        .map_or(Ok(DEFAULT_SAMPLES), |s| s.parse::<usize>());
    let (iterations, count) = match (iterations, count) {
        (Ok(iterations), Ok(count)) if iterations > 0 && count > 0 => (iterations, count),
        _ => {
            eprintln!("Iterations and samples must be positive integers.");
            process::exit(2);
        }
    };
    if cfg!(debug_assertions) {
        eprintln!("Use --release for meaningful timing results.");
        process::exit(2);
    }

    let operation = Transform {
        mask: black_box(0x9e37_79b9_7f4a_7c15),
    };
    // The seed is the first input to the chain; later calls use the previous result.
    // Both paths start with the same value so they perform equivalent work.
    // Its exact value is arbitrary, and it need not be random. Black-boxing it
    // discourages constant-based optimization of the benchmark input.
    let seed = black_box(0x1234_5678_9abc_def0);
    // Warm both code paths before collecting timings. Setup/allocation is untimed.
    for _ in 0..3 {
        let direct = black_box(static_dispatch(&operation, iterations, seed));
        let indirect = black_box(vtable_dispatch(&operation, iterations, seed));
        assert_eq!(direct, indirect);
    }
    let mut direct_samples = Vec::with_capacity(count);
    let mut indirect_samples = Vec::with_capacity(count);
    let mut paired_deltas = Vec::with_capacity(count);
    for sample in 0..count {
        let direct = || static_dispatch(&operation, iterations, seed);
        let indirect = || vtable_dispatch(&operation, iterations, seed);
        // Alternate order to reduce systematic first/second-run bias.
        let (static_result, dynamic_result) = if sample % 2 == 0 {
            (measure(iterations, direct), measure(iterations, indirect))
        } else {
            let dynamic_result = measure(iterations, indirect);
            (measure(iterations, direct), dynamic_result)
        };
        assert_eq!(static_result.1, dynamic_result.1);
        paired_deltas.push(dynamic_result.0 - static_result.0);
        direct_samples.push(static_result.0);
        indirect_samples.push(dynamic_result.0);
    }

    println!("{iterations} calls/sample, {count} samples; nanoseconds per call");
    println!("Variant      Median        Min        Max      Diff %");
    let direct = report("Static", &mut direct_samples, None);
    let indirect = report("Vtable", &mut indirect_samples, Some(direct));
    println!(
        "Vtable delta: {:+.3} ns/call ({:+.1}%)",
        indirect - direct,
        (indirect / direct - 1.0) * 100.0
    );
    println!(
        "Median paired delta: {:+.3} ns/call",
        median(&mut paired_deltas)
    );
    println!("Same non-inlined function body; one dependent call per iteration.");
    println!("One local run; repeat on an idle machine before drawing conclusions.");
}
