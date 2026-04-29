//! Regex parse perf probe — Phase 0 measurement infrastructure for PGEN-RGX-0073.
//!
//! Methodology mirrors the RGX-side `compile_phase_split` example:
//!   - 1000 timed samples per pattern after 50 warmup iterations.
//!   - Release build.
//!   - Each sample includes parser construction + `parse_full_regex()`,
//!     because that's the unit RGX's `parsing::parse_pattern` measures.
//!   - Reports min / p50 / mean / p99 / max nanoseconds per pattern.
//!
//! Cross-check: numbers should align with
//! `/Users/richarddje/Documents/github/rgx/pgen-issues/artifacts/PGEN-RGX-0073/rgx_compile_phase_split.txt`.
//!
//! This is parser-agnostic infrastructure-wise (the harness is generic enough to
//! extend to other grammars), but Phase 0 only exercises the regex grammar
//! because PGEN-RGX-0073 is the bug we're triaging.
//!
//! Usage:
//!   cargo run --release --features generated_parsers --bin regex_perf_probe
//!   cargo run --release --features generated_parsers --bin regex_perf_probe -- --samples 5000 --warmup 200

// Optim #10 candidate: opt-in mimalloc as the global allocator. macOS's
// libsystem_malloc dominates the post-Optim-#9 self-time profile (~30% of
// samples land inside it). Build with `--features mimalloc_perf` to swap
// in mimalloc and measure the impact in isolation. The library itself
// does NOT set a global allocator — embedders pick.
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
                    "regex_perf_probe — measure regex parse time on the PGEN-RGX-0073 8-pattern corpus.\n\nUsage:\n  regex_perf_probe [--samples N] [--warmup N]"
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

#[cfg(all(feature = "generated_parsers"))]
fn time_one_parse(input: &str) -> u64 {
    let start = Instant::now();
    let mut parser = RegexParser::new(
        input,
        pgen::ast_pipeline::runtime_logger_box("regex_perf_probe"),
    );
    let _ = parser.parse_full_regex();
    start.elapsed().as_nanos() as u64
}

#[cfg(not(all(feature = "generated_parsers")))]
fn time_one_parse(_input: &str) -> u64 {
    0
}

/// Phase 2 M3 stage 7: typed-path measurement. Calls the
/// `parse_full_regex_typed` entry-point that dispatches through the
/// per-rule `parse_<rule>_typed` methods. Stages 2-5 emit shape-typed
/// bodies for ~74% of regex grammar rules; the rest fall back to the
/// stage-1 wrapper (legacy + `serde_json::to_value`).
///
/// Gated behind `--features phase_2_typed_emit_probe` because
/// `parse_full_regex_typed` only exists in the regex parser when it
/// has been regenerated with `--inline-annotations`
/// (`make regex_parser_typed`). The default tracked
/// `generated/regex_parser.rs` ships the legacy emit only because
/// stage 4b (annotation-aware typed emit) hasn't landed yet, so the
/// typed path measures ~1.0× legacy on the regex grammar — the hot
/// path (`regex` / `pattern` / `piece`) is annotated and falls back
/// through `serde_json::to_value`.
#[cfg(all(feature = "generated_parsers", feature = "phase_2_typed_emit_probe"))]
fn time_one_parse_typed(input: &str) -> u64 {
    let start = Instant::now();
    let mut parser = RegexParser::new(
        input,
        pgen::ast_pipeline::runtime_logger_box("regex_perf_probe_typed"),
    );
    let _ = parser.parse_full_regex_typed();
    start.elapsed().as_nanos() as u64
}

#[cfg(not(all(feature = "generated_parsers", feature = "phase_2_typed_emit_probe")))]
fn time_one_parse_typed(_input: &str) -> u64 {
    0
}

fn measure(name: &'static str, input: &str, samples: usize, warmup: usize) -> Stats {
    // Warmup
    for _ in 0..warmup {
        let _ = time_one_parse(input);
    }
    // Measure
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

fn measure_typed(name: &'static str, input: &str, samples: usize, warmup: usize) -> Stats {
    for _ in 0..warmup {
        let _ = time_one_parse_typed(input);
    }
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        times.push(time_one_parse_typed(input));
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
    println!("# Regex parse perf probe — PGEN-RGX-0073 baseline");
    println!(
        "# samples={} warmup={} build=release feature=generated_parsers",
        samples, warmup
    );
    println!();
    println!(
        "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10}",
        "pattern", "min (ns)", "p50 (ns)", "mean (ns)", "p99 (ns)", "max (ns)", "samples"
    );
    println!("{}", "-".repeat(102));
    let mut legacy_p50: Vec<(&'static str, u64)> = Vec::with_capacity(PATTERNS.len());
    for (name, input) in PATTERNS {
        let s = measure(name, input, samples, warmup);
        println!(
            "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10}",
            s.name, s.min_ns, s.p50_ns, s.mean_ns, s.p99_ns, s.max_ns, s.samples
        );
        legacy_p50.push((s.name, s.p50_ns));
    }
    println!();
    #[cfg(feature = "phase_2_typed_emit_probe")]
    {
        println!("# Phase 2 M3 stage 7: typed-path measurement via parse_full_regex_typed");
        println!(
            "# (~74% of rules use shape-typed emit at the current codegen; the rest fall back to legacy + serde_json::to_value)"
        );
        println!();
        println!(
            "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10} {:>14}",
            "pattern", "min (ns)", "p50 (ns)", "mean (ns)", "p99 (ns)", "max (ns)", "samples", "vs legacy"
        );
        println!("{}", "-".repeat(118));
        for (idx, (name, input)) in PATTERNS.iter().enumerate() {
            let s = measure_typed(name, input, samples, warmup);
            let legacy_p50_ns = legacy_p50[idx].1 as f64;
            let typed_p50_ns = s.p50_ns as f64;
            let ratio = if typed_p50_ns > 0.0 {
                legacy_p50_ns / typed_p50_ns
            } else {
                0.0
            };
            println!(
                "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10} {:>14}",
                s.name,
                s.min_ns,
                s.p50_ns,
                s.mean_ns,
                s.p99_ns,
                s.max_ns,
                s.samples,
                format!("{:.2}x", ratio)
            );
        }
        println!();
    }
    #[cfg(not(feature = "phase_2_typed_emit_probe"))]
    let _ = legacy_p50; // silence unused-binding warning when typed probe is off
    #[cfg(not(feature = "phase_2_typed_emit_probe"))]
    let _: fn(&'static str, &str, usize, usize) -> Stats = measure_typed;
    println!(
        "# RGX bug-bundle reference (rgx_compile_phase_split.txt, 1000 samples, 50 warmup, Apple M4 Pro):"
    );
    println!(
        "# literal_simple ~407ns p50  digit_sequence ~819ns  character_class ~2.35ms  alternation ~1.02ms"
    );
    println!(
        "# capture_groups ~1.56ms     url_simple ~1.45ms      email_basic ~1.17ms     anchor_complex ~2.70ms"
    );
}
