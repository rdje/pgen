//! Regex allocation-census probe — the MATCH-THEN-BUILD v2 STEP-0 `f_spec`
//! instrument (RGX-0078.5.i.7, slice PGEN-RGX-0078-0092).
//!
//! QUESTION IT ANSWERS: of the allocation traffic a bare (fused-graph) regex
//! parse pays INSIDE the min-metric region, how much builds values that are
//! DOOMED (failed speculation, tournament losers, winner-replay copies) versus
//! values that survive into the committed AST? That doomed share — `f_spec` —
//! is the falsifiable pricing input for the match-then-build lever (defer value
//! construction to the committed walk), which RE-PROFILE #11 priced only as an
//! UNMEASURED band (the value/alloc complex ≈71% × f_spec).
//!
//! WHY exact event counts, not profile samples: the P4-iii refutation showed a
//! SAMPLED alloc share over-pricing a fold ≈5× on the min metric (part of its
//! surface lived after the stopwatch, in teardown). This probe counts every
//! allocator event through a counting `GlobalAlloc` wrapper and SEGMENTS them
//! at exactly the boundaries `regex_perf_probe::time_one_parse` times:
//!   - IN-METRIC:  arena + parser construction + `parse_full_regex()` — the
//!     region the campaign's `elapsed()` stopwatch measures;
//!   - TEARDOWN:   dropping the parse result, parser, and arena (the mass
//!     free of every arena node incl. dead speculative values) — traffic the
//!     min metric NEVER sees, reported separately so no ceiling is priced on it;
//!   - COMMITTED (proxy): `serde_json::to_value(&parsed)` over a fresh parse's
//!     result — a fresh copy of every committed value node, approximating the
//!     committed share of the in-metric build traffic.
//! Derived: `f_spec_allocs = 1 − allocs_committed/allocs_in` (and the bytes
//! twin), plus `frees_in/allocs_in` — mid-parse frees are pure doomed/transient
//! traffic (committed values survive to teardown by construction), a second,
//! proxy-free signal that brackets `f_spec` from below.
//!
//! HONEST PROXY BOUNDS (both directions, named): the serialization proxy
//! OVER-counts committed traffic where the parse borrowed instead of building
//! (`ParseContent::Terminal` is a zero-alloc borrow in-parse but a fresh
//! `String` in the JSON copy) ⇒ `f_spec_allocs` is conservative (a floor);
//! it UNDER-counts where the committed path itself built intermediate copies
//! (winner-replay clones) that MTB would also kill ⇒ the true kill surface is
//! at or above the floor. Counts are exact allocator events, deterministic for
//! a deterministic parser (asserted by `--repeat`, default 2), and
//! build-mode-independent to first order (allocation events are semantic).
//!
//! The canonical `regex_perf_probe` timing instrument is deliberately
//! UNTOUCHED: this sibling bin carries the counting allocator so timing runs
//! stay uninstrumented.
//!
//! Usage:
//!   cargo build --features generated_parsers --bin regex_alloc_census_probe
//!   ./target/debug/regex_alloc_census_probe [--repeat N]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static FREE_CALLS: AtomicU64 = AtomicU64::new(0);
static FREE_BYTES: AtomicU64 = AtomicU64::new(0);

/// System-backed counting allocator. `realloc` counts as one alloc of the new
/// size plus one free of the old size so alloc/free totals stay balanced.
struct CountingAlloc;

// SAFETY: delegates every operation verbatim to `System`; the counters are
// side-effect-only atomics.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        FREE_CALLS.fetch_add(1, Ordering::Relaxed);
        FREE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        FREE_CALLS.fetch_add(1, Ordering::Relaxed);
        FREE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Same corpus as `regex_perf_probe` (the campaign's 8-pattern bench).
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Snap {
    alloc_calls: u64,
    alloc_bytes: u64,
    free_calls: u64,
    free_bytes: u64,
}

fn snap() -> Snap {
    Snap {
        alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
        alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        free_calls: FREE_CALLS.load(Ordering::Relaxed),
        free_bytes: FREE_BYTES.load(Ordering::Relaxed),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Segment {
    alloc_calls: u64,
    alloc_bytes: u64,
    free_calls: u64,
    free_bytes: u64,
}

fn delta(a: Snap, b: Snap) -> Segment {
    Segment {
        alloc_calls: b.alloc_calls - a.alloc_calls,
        alloc_bytes: b.alloc_bytes - a.alloc_bytes,
        free_calls: b.free_calls - a.free_calls,
        free_bytes: b.free_bytes - a.free_bytes,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Row {
    accepted: bool,
    in_metric: Segment,
    teardown: Segment,
    committed: Segment,
}

#[cfg(feature = "generated_parsers")]
fn census_one(input: &str) -> Row {
    use pgen::generated_parsers::regex::RegexParser;

    // Run 1 — segment the exact region `regex_perf_probe::time_one_parse`
    // times (arena + parser construction + parse; NO diagnostic consumer is
    // touched, so the parse stays BARE = the fused cascade graph), then the
    // post-stopwatch teardown.
    let s0 = snap();
    let accepted;
    let s1;
    {
        let node_arena = pgen::ast_pipeline::NodeArena::new();
        let mut parser = RegexParser::new(
            input,
            &node_arena,
            pgen::ast_pipeline::runtime_logger_box("regex_alloc_census_probe"),
        );
        let parsed = parser.parse_full_regex();
        s1 = snap(); // the `elapsed()` boundary
        accepted = parsed.is_ok();
    } // parsed + parser + arena drop here (the mass free)
    let s2 = snap();

    // Run 2 — the committed-value proxy: a fresh (uncounted) parse, then a
    // counted `serde_json::to_value` over its result — a fresh copy of every
    // committed value node.
    let node_arena = pgen::ast_pipeline::NodeArena::new();
    let mut parser = RegexParser::new(
        input,
        &node_arena,
        pgen::ast_pipeline::runtime_logger_box("regex_alloc_census_probe"),
    );
    let committed = match parser.parse_full_regex() {
        Ok(parsed) => {
            let s3 = snap();
            let value = serde_json::to_value(&parsed)
                .expect("committed parse tree must serialize");
            let s4 = snap();
            drop(value);
            delta(s3, s4)
        }
        Err(_) => Segment {
            alloc_calls: 0,
            alloc_bytes: 0,
            free_calls: 0,
            free_bytes: 0,
        },
    };

    Row {
        accepted,
        in_metric: delta(s0, s1),
        teardown: delta(s1, s2),
        committed,
    }
}

#[cfg(not(feature = "generated_parsers"))]
fn census_one(_input: &str) -> Row {
    panic!("regex_alloc_census_probe requires --features generated_parsers");
}

fn parse_args() -> usize {
    let mut repeat = 2usize;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repeat" => {
                i += 1;
                repeat = args[i].parse().expect("--repeat expects integer");
            }
            "-h" | "--help" => {
                eprintln!(
                    "regex_alloc_census_probe — count allocator events per bare regex parse,\nsegmented in-metric / teardown / committed-proxy (the MTB f_spec instrument).\n\nUsage:\n  regex_alloc_census_probe [--repeat N]"
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
    repeat.max(1)
}

fn main() {
    let repeat = parse_args();
    println!("# Regex allocation-census probe — MTB v2 STEP-0 f_spec instrument (RGX-0078.5.i.7)");
    println!("# segments: IN = arena+parser+parse (the min-metric region) | TD = post-stopwatch teardown | COMM = serde_json::to_value committed proxy");
    println!("# repeat={} (counts asserted identical across repeats)", repeat);
    println!();

    // Warmup: one full census of every pattern so lazily-initialized global
    // state (logger, caches) is paid before any measured run.
    for (_, input) in PATTERNS {
        let _ = census_one(input);
    }

    println!(
        "{:<18} {:>3} {:>9} {:>10} {:>9} {:>9} {:>9} {:>9} {:>10} {:>8} {:>8} {:>8}",
        "pattern",
        "ok",
        "in_alloc",
        "in_bytes",
        "in_free",
        "td_alloc",
        "td_free",
        "co_alloc",
        "co_bytes",
        "f_alloc",
        "f_bytes",
        "free_in"
    );
    println!("{}", "-".repeat(126));

    let mut det_ok = true;
    for (name, input) in PATTERNS {
        let first = census_one(input);
        for r in 1..repeat {
            let again = census_one(input);
            if again != first {
                det_ok = false;
                eprintln!(
                    "DETERMINISM VIOLATION on '{}' repeat {}: {:?} != {:?}",
                    name, r, again, first
                );
            }
        }
        let f_alloc = 1.0
            - first.committed.alloc_calls as f64 / first.in_metric.alloc_calls.max(1) as f64;
        let f_bytes = 1.0
            - first.committed.alloc_bytes as f64 / first.in_metric.alloc_bytes.max(1) as f64;
        let free_in_share =
            first.in_metric.free_calls as f64 / first.in_metric.alloc_calls.max(1) as f64;
        println!(
            "{:<18} {:>3} {:>9} {:>10} {:>9} {:>9} {:>9} {:>9} {:>10} {:>7.1}% {:>7.1}% {:>7.1}%",
            name,
            if first.accepted { "y" } else { "N" },
            first.in_metric.alloc_calls,
            first.in_metric.alloc_bytes,
            first.in_metric.free_calls,
            first.teardown.alloc_calls,
            first.teardown.free_calls,
            first.committed.alloc_calls,
            first.committed.alloc_bytes,
            f_alloc * 100.0,
            f_bytes * 100.0,
            free_in_share * 100.0,
        );
    }

    println!();
    println!("# f_alloc/f_bytes = 1 - committed_proxy/in_metric (proxy over-counts committed borrows -> these are FLOORS for f_spec)");
    println!("# free_in = frees inside the min-metric region / in-metric allocs (mid-parse frees are doomed/transient traffic only)");
    if det_ok {
        println!("# determinism: counts identical across {} repeats for every pattern", repeat);
    } else {
        println!("# DETERMINISM VIOLATION observed — see stderr");
        std::process::exit(1);
    }
}
