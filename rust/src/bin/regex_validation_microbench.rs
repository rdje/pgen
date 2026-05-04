//! Microbench: measure validate_regex_compile_contract cost on the
//! PGEN-RGX-0073/0078 8-pattern bench corpus. Used to decide whether
//! short-circuiting the 11 validation passes is worth Optim #16.

#[cfg(feature = "mimalloc_perf")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use pgen::regex_compile_validation::validate_regex_compile_contract;
use std::time::Instant;

const PATTERNS: &[(&str, &str)] = &[
    ("literal_simple", "test"),
    ("digit_sequence", r"\d{3}-\d{2}-\d{4}"),
    (
        "character_class",
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
    ),
    ("alternation", "cat|dog|bird"),
    ("capture_groups", r"(\d{4})-(\d{2})-(\d{2})"),
    ("url_simple", r"https?://\S+"),
    ("email_basic", r"\b\w+@\w+\.\w+\b"),
    ("anchor_complex", r"^(\d+)\s+(?P<word>\w+)\s+(?:foo|bar)$"),
];

const SAMPLES: usize = 5000;
const WARMUP: usize = 200;

fn main() {
    println!("# validate_regex_compile_contract microbench");
    println!(
        "# samples={} warmup={} build=release feature=mimalloc_perf",
        SAMPLES, WARMUP
    );
    println!();
    println!(
        "{:<22} {:>14} {:>14} {:>14} {:>14}",
        "pattern", "min (ns)", "p50 (ns)", "mean (ns)", "p99 (ns)"
    );
    println!("{}", "-".repeat(80));
    for (name, pattern) in PATTERNS {
        for _ in 0..WARMUP {
            let _ = validate_regex_compile_contract(pattern);
        }
        let mut times = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let t0 = Instant::now();
            let _ = validate_regex_compile_contract(pattern);
            times.push(t0.elapsed().as_nanos() as u64);
        }
        times.sort_unstable();
        let mean = (times.iter().sum::<u64>() as f64 / times.len() as f64) as u64;
        println!(
            "{:<22} {:>14} {:>14} {:>14} {:>14}",
            name,
            times[0],
            times[times.len() / 2],
            mean,
            times[(times.len() as f64 * 0.99) as usize]
        );
    }
}
