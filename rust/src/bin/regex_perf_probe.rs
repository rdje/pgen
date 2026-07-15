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

// mimalloc as the global allocator — LANDED (RGX-0078.5.i.7 `-0097`, the
// recommended-production allocator). macOS's libsystem_malloc + `_xzm_free`
// dominate the value/alloc self-time profile; MEASURED (fat-LTO alternated
// 5×2000 geomean-of-mins): mimalloc is **−29.2%** vs system malloc, capturing
// 73% of the never-free-arena ceiling (part (a), below) from a one-line swap,
// deterministic, correctness-neutral by construction (the `.4.a` fat-LTO
// build-profile precedent). `--features mimalloc_perf` is now the STANDARD
// closure-bench configuration and the campaign's `.5` baseline probe. The
// LIBRARY still does NOT set a global allocator — that stays an embedder
// decision (a library must not impose one); the regex integration contract
// recommends a mimalloc-class allocator to embedders (RGX).
#[cfg(feature = "mimalloc_perf")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// RGX-0078.5.i.7 STEP-0 part (a): the NEVER-FREE (reset-per-sample) bump-arena
// global allocator — a MEASUREMENT-ONLY ceiling instrument (build with
// `--features never_free_arena_perf`). It prices the FALSIFIABLE LOWER BOUND of
// the value/alloc lever: RE-PROFILE #11 attributes ~41% of self-time to the
// allocator and ~21.5% to `_xzm_free` (freeing dead value trees). Under this
// allocator every in-parse `malloc` is a pointer bump and every `free` is a
// no-op, so BOTH costs collapse to ~zero; the resulting `parse_full_regex` time
// is the fastest the match-then-build lever (MTB-B doomed-value elision) could
// ever reach on the value/alloc axis. If it barely moves the metric, the
// value/alloc profile is misattributed in TIME and the `.5` lever must pivot to
// the ~29% matching/dispatch compute instead.
//
// WHY reset-per-sample rather than a pure never-free arena: a bare parse
// allocates 193KB–438KB in-metric (regex_alloc_census_probe), i.e. ~48–110
// FRESH (zero-faulted) pages. Never actually reclaiming them would pay ~48–110
// minor page faults per parse (~50–110µs) — swamping the ~14µs parse and giving
// a nonsense (much SLOWER) reading that measures page-fault cost, not allocator
// cost. Instead the bump cursor is RESET to a mark (taken once, after warmup)
// BEFORE each timed sample: committed pages are retained and reused, so after
// warmup no new page faults occur and the working set stays hot in cache. The
// reset happens OUTSIDE the stopwatch, so the timed region still measures pure
// bump-alloc + no-op-free. Safety: nothing allocated during a measured parse
// survives the reset (`time_one_parse` drops the arena/parser/result before it
// returns; the `times` Vec is reserved BELOW the mark and never reallocates),
// and warmup initializes all lazy statics below the mark.
#[cfg(feature = "never_free_arena_perf")]
mod never_free_arena {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::UnsafeCell;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// System-backed backing chunk size (address reserved; pages committed on
    /// touch). Large enough that a full run's below-mark retention (warmup +
    /// per-pattern marks, ~175 MB across the 8-pattern corpus) plus one parse's
    /// transients stay in a handful of chunks.
    const CHUNK: usize = 512 * 1024 * 1024;
    /// A reset mark must have at least this much room left in its chunk, so one
    /// parse's ≤438KB of transients never crosses into a fresh chunk (which
    /// would re-fault pages every sample and defeat the point of the reset).
    const RESET_HEADROOM: usize = 32 * 1024 * 1024;

    struct State {
        cursor: *mut u8,
        end: *mut u8,
    }

    pub struct NeverFreeBump {
        lock: AtomicBool,
        state: UnsafeCell<State>,
    }

    // SAFETY: every access to `state` is serialized by the `lock` spinlock. The
    // probe's hot loop is single-threaded, so the lock is uncontended.
    unsafe impl Sync for NeverFreeBump {}

    impl NeverFreeBump {
        pub const fn new() -> Self {
            NeverFreeBump {
                lock: AtomicBool::new(false),
                state: UnsafeCell::new(State {
                    cursor: std::ptr::null_mut(),
                    end: std::ptr::null_mut(),
                }),
            }
        }

        #[inline]
        fn acquire(&self) {
            while self
                .lock
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                std::hint::spin_loop();
            }
        }

        #[inline]
        fn release(&self) {
            self.lock.store(false, Ordering::Release);
        }

        /// Reserve a fresh backing chunk from `System` (independent of this
        /// global allocator, so no re-entrancy). SAFETY: caller holds the lock.
        unsafe fn new_chunk(st: &mut State, min_bytes: usize) {
            let size = min_bytes.max(CHUNK);
            let layout = Layout::from_size_align(size, 4096).expect("valid chunk layout");
            let base = unsafe { System.alloc(layout) };
            assert!(
                !base.is_null(),
                "never-free arena: System chunk allocation failed ({size} bytes)"
            );
            st.cursor = base;
            st.end = unsafe { base.add(size) };
        }

        /// Freeze the current cursor as a reset mark, guaranteeing at least
        /// `RESET_HEADROOM` bytes remain in its chunk so a parse's transients
        /// never leave it (keeping reused pages hot).
        pub fn mark(&self) -> usize {
            self.acquire();
            let st = unsafe { &mut *self.state.get() };
            let room = (st.end as usize).saturating_sub(st.cursor as usize);
            if st.cursor.is_null() || room < RESET_HEADROOM {
                unsafe { Self::new_chunk(st, RESET_HEADROOM) };
            }
            let m = st.cursor as usize;
            self.release();
            m
        }

        /// Reset the bump cursor back to a mark, retaining (not freeing) the
        /// committed pages so the next sample reuses them.
        pub fn reset_to(&self, mark: usize) {
            self.acquire();
            let st = unsafe { &mut *self.state.get() };
            st.cursor = mark as *mut u8;
            self.release();
        }
    }

    // SAFETY: bump-pointer allocation within System-backed chunks; `dealloc` is
    // a deliberate no-op (the arena is reset per-sample, never per-object). The
    // default `realloc`/`alloc_zeroed` route through `alloc`+`dealloc`.
    unsafe impl GlobalAlloc for NeverFreeBump {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            self.acquire();
            let st = unsafe { &mut *self.state.get() };
            let align = layout.align();
            let size = layout.size();
            loop {
                let cur = st.cursor as usize;
                let aligned = cur.wrapping_add(align - 1) & !(align - 1);
                let next = aligned.wrapping_add(size);
                if !st.cursor.is_null() && aligned >= cur && next <= st.end as usize {
                    st.cursor = next as *mut u8;
                    self.release();
                    return aligned as *mut u8;
                }
                // Uninitialized or insufficient room: grow, then retry.
                unsafe { Self::new_chunk(st, size.saturating_add(align)) };
            }
        }

        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // No-op: the arena is reset per-sample (see the module comment).
        }
    }
}

#[cfg(feature = "never_free_arena_perf")]
#[global_allocator]
static GLOBAL: never_free_arena::NeverFreeBump = never_free_arena::NeverFreeBump::new();

#[cfg(all(feature = "mimalloc_perf", feature = "never_free_arena_perf"))]
compile_error!(
    "mimalloc_perf and never_free_arena_perf both set #[global_allocator]; enable at most one"
);

/// Freeze the bump-arena cursor after warmup (no-op unless the never-free arena
/// is the active allocator). Measured samples reset here so committed pages stay
/// hot — the reset is outside the timed region.
#[cfg(feature = "never_free_arena_perf")]
#[inline]
fn arena_mark() -> usize {
    GLOBAL.mark()
}

#[cfg(feature = "never_free_arena_perf")]
#[inline]
fn arena_reset_to(mark: usize) {
    GLOBAL.reset_to(mark);
}

#[cfg(not(feature = "never_free_arena_perf"))]
#[inline]
fn arena_mark() -> usize {
    0
}

#[cfg(not(feature = "never_free_arena_perf"))]
#[inline]
fn arena_reset_to(_mark: usize) {}

use std::time::Instant;

#[cfg(feature = "generated_parsers")]
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

#[cfg(feature = "generated_parsers")]
fn time_one_parse(input: &str) -> u64 {
    let start = Instant::now();
    let node_arena = pgen::ast_pipeline::NodeArena::new();
    let mut parser = RegexParser::new(
        input,
        &node_arena,
        pgen::ast_pipeline::runtime_logger_box("regex_perf_probe"),
    );
    let _ = parser.parse_full_regex();
    start.elapsed().as_nanos() as u64
}

#[cfg(not(feature = "generated_parsers"))]
fn time_one_parse(_input: &str) -> u64 {
    0
}

/// Under the never-free arena, assert every pattern still parse-ACCEPTS before
/// timing, so any arena/reset corruption surfaces as a loud abort rather than
/// silently biasing the numbers. No-op in every other build (the timed loop is
/// untouched — this runs once, before warmup).
#[cfg(all(feature = "generated_parsers", feature = "never_free_arena_perf"))]
fn verify_arena_accepts() {
    for (name, input) in PATTERNS {
        let node_arena = pgen::ast_pipeline::NodeArena::new();
        let mut parser = RegexParser::new(
            input,
            &node_arena,
            pgen::ast_pipeline::runtime_logger_box("regex_perf_probe:verify"),
        );
        assert!(
            parser.parse_full_regex().is_ok(),
            "never-free arena corrupted the parse of '{name}' (expected accept)"
        );
    }
}

#[cfg(not(all(feature = "generated_parsers", feature = "never_free_arena_perf")))]
fn verify_arena_accepts() {}

fn measure(name: &'static str, input: &str, samples: usize, warmup: usize) -> Stats {
    // Reserve the results buffer BEFORE warmup so — under the never-free arena —
    // it sits below the reset mark and its backing store is never reclaimed or
    // reallocated (exactly `samples` pushes, capacity `samples`).
    let mut times = Vec::with_capacity(samples);
    // Warmup (also initializes all lazy statics below the mark).
    for _ in 0..warmup {
        let _ = time_one_parse(input);
    }
    // Freeze the arena cursor (no-op unless the never-free arena is active).
    let mark = arena_mark();
    // Measure
    for _ in 0..samples {
        // Retain-and-reuse committed pages across samples (outside the timed
        // region, so the stopwatch still sees pure bump-alloc + no-op-free).
        arena_reset_to(mark);
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
    verify_arena_accepts();
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
    for (name, input) in PATTERNS {
        let s = measure(name, input, samples, warmup);
        println!(
            "{:<18} {:>14} {:>14} {:>14} {:>14} {:>14} {:>10}",
            s.name, s.min_ns, s.p50_ns, s.mean_ns, s.p99_ns, s.max_ns, s.samples
        );
    }
    println!();
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
