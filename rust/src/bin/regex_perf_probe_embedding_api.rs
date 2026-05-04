//! Apples-to-apples PGEN-RGX-0078 verification probe.
//!
//! Mirrors regex_perf_probe.rs but measures via `parse_grammar_profile_named`
//! (the embedding-API path RGX measured), not the direct RegexParser path.
//! Used to quantify the path-overhead delta vs the direct probe.

#[cfg(feature = "mimalloc_perf")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

fn percentile(sorted: &[u64], p: f64) -> u64 {
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn measure(input: &str) -> (u64, u64, u64, u64, u64) {
    for _ in 0..WARMUP {
        let _ = pgen::embedding_api::parse_grammar_profile_named(
            "regex", "regex_default", input,
        );
    }
    let mut times = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let t0 = Instant::now();
        let outcome = pgen::embedding_api::parse_grammar_profile_named(
            "regex", "regex_default", input,
        );
        let dt = t0.elapsed().as_nanos() as u64;
        assert!(matches!(
            outcome.status,
            pgen::embedding_api::ParseStatus::Success
        ));
        times.push(dt);
    }
    times.sort_unstable();
    let mean = (times.iter().sum::<u64>() as f64 / times.len() as f64) as u64;
    (
        times[0],
        percentile(&times, 0.50),
        mean,
        percentile(&times, 0.99),
        *times.last().unwrap(),
    )
}

fn main() {
    println!(
        "# regex_perf_probe_embedding_api — parse_grammar_profile_named path"
    );
    println!(
        "# samples={} warmup={} build=release feature=generated_parsers+mimalloc_perf",
        SAMPLES, WARMUP
    );
    println!();
    println!(
        "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14}",
        "pattern", "min (ns)", "p50 (ns)", "mean (ns)", "p99 (ns)", "max (ns)"
    );
    println!("{}", "-".repeat(92));
    for (name, input) in PATTERNS {
        let (mn, p50, mean, p99, mx) = measure(input);
        println!(
            "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14}",
            name, mn, p50, mean, p99, mx
        );
    }
}
