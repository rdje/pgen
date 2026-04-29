//! Regex typed-path perf probe — slice 4 measurement infrastructure for
//! PGEN-RGX-0073.
//!
//! Sibling of [`regex_perf_probe`][regex_perf_probe], with one
//! difference: this probe calls the regex parser hook's typed entry
//! method `parse_regex_typed()` instead of the legacy
//! `parse_full_regex()`. Same 8-pattern PGEN-RGX-0073 bug corpus, same
//! warmup / sampling methodology, same statistics.
//!
//! [regex_perf_probe]: ./regex_perf_probe.rs
//!
//! # What this measures
//!
//! Each timed sample includes:
//!   - `RegexParser::new(...)` construction
//!   - `parser.parse_regex_typed()` — which today (slice 2 passthrough
//!     body) is `self.parse_regex()?.content.to_json_value()`.
//!
//! The legacy probe measures `parse_full_regex()` which returns a
//! `ParseNode<'input>`. The typed probe measures the JSON-producing
//! path. The two are NOT apples-to-apples — the typed path does
//! strictly more work today (full ParseNode build + JSON conversion).
//! That is exactly the gap subsequent shape-typed-emit slices close:
//! each slice replaces specific rules' typed bodies with direct
//! `serde_json::Value` construction (preserving
//! `with_semantic_runtime_rule_transaction` + `memoized_call`
//! semantics), bringing the typed-path numbers down toward — and
//! eventually below — the legacy probe's numbers.
//!
//! # Why probe both
//!
//! Without an apples-to-apples typed-path baseline, "shape-typed emit
//! is faster" claims are unverifiable. The legacy probe's numbers are
//! pinned to the RGX bug bundle's `compile_phase_split` reference and
//! aren't affected by the hook (they call legacy methods that exist
//! whether or not the hook is registered). The typed probe's numbers
//! are the measurement target that future slices have to move.
//!
//! # Requirements to run
//!
//! `parse_regex_typed` only exists when the regex parser was
//! regenerated with the regex parser hook registered (i.e. with
//! `--enable-parser-hooks`). The maintained
//! `make regex_typed_perf_probe` target handles regen + build + run +
//! restore in one shot, mirroring the differential gate's procedure.
//!
//! Direct cargo invocation requires the parser to already have been
//! regenerated with the hook registered:
//!
//! ```sh
//! cargo run --release \
//!   --features generated_parsers,regex_typed_perf_probe \
//!   --bin regex_typed_perf_probe
//! ```

// Optim #10 carry-over: same opt-in mimalloc allocator switch as the
// legacy probe, so apples-to-apples comparisons can be done with the
// same allocator on both sides.
#[cfg(feature = "mimalloc_perf")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time::Instant;

#[cfg(all(feature = "generated_parsers"))]
use pgen::generated_parsers::regex::RegexParser;

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

struct Stats {
    name: &'static str,
    samples: usize,
    min_ns: u64,
    p50_ns: u64,
    mean_ns: u64,
    p99_ns: u64,
    max_ns: u64,
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn parse_args() -> (usize, usize) {
    let mut samples = 1000usize;
    let mut warmup = 50usize;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--samples" => {
                i += 1;
                samples = args[i].parse().expect("--samples expects integer");
            }
            "--warmup" => {
                i += 1;
                warmup = args[i].parse().expect("--warmup expects integer");
            }
            "-h" | "--help" => {
                eprintln!(
                    "regex_typed_perf_probe — measure regex parse_regex_typed() time on the PGEN-RGX-0073 8-pattern corpus.\n\nUsage:\n  regex_typed_perf_probe [--samples N] [--warmup N]"
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {}", other);
                std::process::exit(2);
            }
        }
        i += 1;
    }
    (samples, warmup)
}

#[cfg(all(feature = "generated_parsers", feature = "regex_typed_perf_probe"))]
fn time_one_parse(input: &str) -> u64 {
    let start = Instant::now();
    let mut parser = RegexParser::new(
        input,
        pgen::ast_pipeline::runtime_logger_box("regex_typed_perf_probe"),
    );
    let _ = parser.parse_regex_typed();
    start.elapsed().as_nanos() as u64
}

#[cfg(not(all(feature = "generated_parsers", feature = "regex_typed_perf_probe")))]
fn time_one_parse(_input: &str) -> u64 {
    0
}

fn measure(name: &'static str, input: &str, samples: usize, warmup: usize) -> Stats {
    for _ in 0..warmup {
        let _ = time_one_parse(input);
    }
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        times.push(time_one_parse(input));
    }
    times.sort_unstable();
    let mean_ns = (times.iter().sum::<u64>() as f64 / times.len() as f64) as u64;
    Stats {
        name,
        samples,
        min_ns: times[0],
        p50_ns: percentile(&times, 0.50),
        mean_ns,
        p99_ns: percentile(&times, 0.99),
        max_ns: *times.last().unwrap(),
    }
}

fn main() {
    let (samples, warmup) = parse_args();
    println!("# Regex typed-path perf probe — PGEN-RGX-0073 typed baseline");
    println!(
        "# samples={} warmup={} build=release feature=generated_parsers,regex_typed_perf_probe",
        samples, warmup
    );
    println!(
        "# unit measured: RegexParser::new + parser.parse_regex_typed() (returns serde_json::Value)"
    );
    println!(
        "# NOTE: today's hook body delegates to legacy parse_regex + ParseContent::to_json_value(),"
    );
    println!(
        "#       so this measurement includes JSON conversion the legacy probe does NOT include."
    );
    println!(
        "#       Subsequent shape-typed-emit slices replace per-rule bodies with direct Value"
    );
    println!(
        "#       construction, narrowing and eventually beating the legacy probe's numbers."
    );
    println!();
    println!(
        "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10}",
        "pattern", "min (ns)", "p50 (ns)", "mean (ns)", "p99 (ns)", "max (ns)", "samples"
    );
    println!("{}", "-".repeat(102));
    for (name, input) in PATTERNS {
        let s = measure(name, input, samples, warmup);
        println!(
            "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10}",
            s.name, s.min_ns, s.p50_ns, s.mean_ns, s.p99_ns, s.max_ns, s.samples
        );
    }
    println!();
    println!(
        "# Compare against `make regex_perf_probe` for the legacy parse_full_regex() baseline."
    );
}
