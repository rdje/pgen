//! Parse allocation-census probe — the ALLOCATOR-SIDE leg of the parse-footprint
//! attribution (`ENGINE-UNIVERSAL-SERVICES.67`(b) LEG 4).
//!
//! QUESTION IT ANSWERS. Legs 1-3 measured the AST side of a parse from INSIDE the
//! engine: arena items, the chunks holding them, and the child-`Vec`/`String` heap
//! each node owns. That accounts for **15.6-36.3 %** of the floor-subtracted peak
//! RSS. The remaining ~64-84 % has four named candidates and no measurement —
//! the memo maps, the derivation tape, transient collection `Vec`s, and allocator
//! overhead/fragmentation. This probe measures the first three as a GROUP and
//! splits them from the fourth, by counting the allocator instead of asking the
//! parser.
//!
//! ⛔⛔ **THE QUANTITY, NAMED BEFORE THE NUMBER** (`-0127`'s carry-forward: a
//! pre-registration inherits the frame of the question it was written in, so name
//! which instrument reads the quantity and whether it reads the same thing the
//! DENOMINATOR is made of). This probe reads **REQUESTED LIVE BYTES** — the sum of
//! `Layout::size()` over every allocation the process has not yet freed. That is an
//! ALLOCATION quantity. `.67`'s denominator, floor-subtracted peak RSS, is a
//! RESIDENCY quantity. They are NOT the same thing and leg 3 measured the gap
//! between them at 1 008 MB of allocation costing 32 768 B of RSS. So:
//!   - the ATTRIBUTION here is allocation-side throughout — every term of the
//!     split, arena included, is a requested-bytes term, so the denominator and
//!     the numerators are made of the same stuff;
//!   - the comparison of peak live bytes against peak RSS is reported as a
//!     CROSS-QUANTITY gap and labelled as one. It is the measurement `-0121`
//!     wanted from the mimalloc swap (*"separates FRAGMENTATION from LIVE DATA"*)
//!     and it measures the quantity directly rather than inferring it from the
//!     behaviour of a second allocator.
//!
//! WHY A SEPARATE BIN. The same reason `arena_chunk_oracle_probe` and
//! `regex_alloc_census_probe` are separate bins: a `#[global_allocator]` is a
//! whole-program property, and the canonical `parseability_probe` — which every
//! tracked corpus number in this repository is measured with — must stay
//! uninstrumented and byte-identical.
//!
//! ⛔ **THE ARM IS VERIFIED BEFORE IT IS MEASURED, NOT AFTER** (`-0123`: a
//! `--features mimalloc_perf` build of `parseability_probe` is NOT a mimalloc arm,
//! because that bin never declares `#[global_allocator]`; measuring it would have
//! produced a confident false ruling from a silently inert arm). This probe
//! REFUSES to report if its own counters read zero at entry to `main`, which they
//! cannot if the counting allocator is actually installed.
//!
//! Usage:
//!   cargo build --release --features generated_parsers --bin parse_alloc_census_probe
//!   PGEN_REPORT_PARSE_FOOTPRINT=1 ./target/release/parse_alloc_census_probe \
//!       systemverilog <file> [--profile sv_2017]
//!
//! Running it with `PGEN_REPORT_PARSE_FOOTPRINT=1` is the point: the engine's arena
//! + owned-heap census and this allocator census then describe **the same parse in
//! the same process**, so the subtraction needs no pairing assumption between two
//! runs.
//!
//! ⛔ **AND THE SUBTRACTION IS ONLY EXACT IF THE PEAK IS WHERE THE AST IS BIGGEST**
//! — see [`PEAK_AT_ALLOC_CALLS`]. The AST legs are monotone and are reported at end
//! of parse; `peak_live − ast` is therefore the non-AST live bytes exactly when the
//! live high-water is reached at the end of the allocating phase, and a LOWER bound
//! on them otherwise. The probe reports the counters that decide which, instead of
//! leaving the reader to assume the flattering one.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

/// One bucket per power of two. `class(s)` for `s` in `[2^(k-1), 2^k)` is `k`, so
/// 48 buckets cover every allocation up to 128 TiB — far past anything a parse can
/// request on a host that has not already died.
const CLASSES: usize = 48;

const ZERO: AtomicU64 = AtomicU64::new(0);

static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static FREE_CALLS: AtomicU64 = AtomicU64::new(0);
static FREE_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

static LIVE_CLASS_BYTES: [AtomicU64; CLASSES] = [ZERO; CLASSES];
static LIVE_CLASS_COUNT: [AtomicU64; CLASSES] = [ZERO; CLASSES];

/// The near-peak snapshot of the two arrays above, plus the live total at the
/// instant it was taken. Copying 96 counters on every new high-water mark would
/// cost O(CLASSES) per allocation through the whole growth phase, so the snapshot
/// is taken only when the live total beats the SNAPSHOT's total by
/// [`snapshot_hysteresis`] bytes. ⇒ `SNAP_LIVE` is within one hysteresis step of
/// `PEAK_LIVE_BYTES`, and the gap is REPORTED rather than assumed negligible.
static SNAP_LIVE: AtomicU64 = AtomicU64::new(0);
static SNAP_CLASS_BYTES: [AtomicU64; CLASSES] = [ZERO; CLASSES];
static SNAP_CLASS_COUNT: [AtomicU64; CLASSES] = [ZERO; CLASSES];
static SNAP_TAKEN: AtomicU64 = AtomicU64::new(0);
static SNAP_HYSTERESIS: AtomicU64 = AtomicU64::new(1 << 16);

/// ⭐⭐ **THESE THREE DECIDE WHETHER THE ATTRIBUTION IS EXACT OR ONE-SIDED, AND
/// THAT IS WHY THEY ARE HERE RATHER THAN REASONED ABOUT AFTERWARDS.** The AST
/// legs (arena chunks + owned heap) are MONOTONE — `typed_arena` frees nothing
/// until it drops — so their reported values are their maxima, taken at the end
/// of the parse. If the live high-water `peak_live` is also reached at the end
/// of the allocating phase, then the AST's value AT THE PEAK equals its reported
/// value and `peak_live − ast` is the non-AST live bytes EXACTLY. If the peak is
/// reached earlier, the AST was smaller then, and `peak_live − ast` is only a
/// LOWER bound on the non-AST share. The difference is not cosmetic — it decides
/// whether "the residual is a minority" is provable — so the alloc/free call
/// counts at the instant the peak was last set are recorded and the leaf reads
/// them rather than assuming monotonicity.
static PEAK_AT_ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static PEAK_AT_FREE_CALLS: AtomicU64 = AtomicU64::new(0);
static PEAK_AT_ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

#[inline]
fn class_of(size: usize) -> usize {
    // `size.max(1)` so a zero-sized request lands in class 1 rather than 0; class 0
    // then means "never used", which keeps the printed table honest.
    (usize::BITS - size.max(1).leading_zeros()) as usize
}

struct CountingAlloc;

// SAFETY: every operation delegates verbatim to `System`; the counters are
// side-effect-only atomics and the snapshot writes touch only static storage, so
// nothing here can allocate and re-enter.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note_alloc(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note_alloc(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A `realloc` is one free of the old size plus one alloc of the new, in
        // that order: the growth of a `Vec` must not be charged as if both buffers
        // were live at once, because they are not.
        note_free(layout.size());
        note_alloc(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        note_free(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[inline]
fn note_alloc(size: usize) {
    let k = class_of(size);
    ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
    ALLOC_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_BYTES[k].fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_COUNT[k].fetch_add(1, Ordering::Relaxed);
    let live = LIVE_BYTES.fetch_add(size as u64, Ordering::Relaxed) + size as u64;
    if live > PEAK_LIVE_BYTES.load(Ordering::Relaxed) {
        PEAK_LIVE_BYTES.store(live, Ordering::Relaxed);
        PEAK_AT_ALLOC_CALLS.store(ALLOC_CALLS.load(Ordering::Relaxed), Ordering::Relaxed);
        PEAK_AT_FREE_CALLS.store(FREE_CALLS.load(Ordering::Relaxed), Ordering::Relaxed);
        PEAK_AT_ALLOC_BYTES.store(ALLOC_BYTES.load(Ordering::Relaxed), Ordering::Relaxed);
        let snap = SNAP_LIVE.load(Ordering::Relaxed);
        if live >= snap.saturating_add(SNAP_HYSTERESIS.load(Ordering::Relaxed)) {
            let snapshot = SNAP_CLASS_BYTES
                .iter()
                .zip(SNAP_CLASS_COUNT.iter())
                .zip(LIVE_CLASS_BYTES.iter().zip(LIVE_CLASS_COUNT.iter()));
            for ((snap_bytes, snap_count), (live_bytes, live_count)) in snapshot {
                snap_bytes.store(live_bytes.load(Ordering::Relaxed), Ordering::Relaxed);
                snap_count.store(live_count.load(Ordering::Relaxed), Ordering::Relaxed);
            }
            SNAP_LIVE.store(live, Ordering::Relaxed);
            SNAP_TAKEN.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[inline]
fn note_free(size: usize) {
    let k = class_of(size);
    FREE_CALLS.fetch_add(1, Ordering::Relaxed);
    FREE_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_BYTES[k].fetch_sub(size as u64, Ordering::Relaxed);
    LIVE_CLASS_COUNT[k].fetch_sub(1, Ordering::Relaxed);
    LIVE_BYTES.fetch_sub(size as u64, Ordering::Relaxed);
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// ⛔ The arm check. If the counting allocator were not installed — the `-0123`
/// failure, where a build flag was believed to have swapped the allocator and had
/// not — these counters would still read zero after Rust's own startup and after
/// reading `argv`, both of which allocate. Refusing here is what makes every number
/// below a measurement of the arm it claims to be.
fn arm_verdict(calls: u64) -> Result<u64, String> {
    if calls == 0 {
        return Err(
            "ARM NOT VERIFIED: the counting #[global_allocator] recorded 0 allocations before \
             main() did any work. It is not installed, and every figure this probe could print \
             would be a measurement of nothing. Refusing rather than reporting zeros."
                .to_string(),
        );
    }
    Ok(calls)
}

fn verify_arm() -> Result<u64, String> {
    arm_verdict(ALLOC_CALLS.load(Ordering::Relaxed))
}

/// ⛔ THE RED CONTROL FOR THE ARM CHECK. A guard nobody has watched fail is a
/// guard nobody has tested, and `-0123` is in this leaf precisely because a build
/// believed to have swapped its allocator had not. `--red-control-arm` feeds the
/// verdict the value it would read if the `#[global_allocator]` were absent and
/// REQUIRES a refusal: exit 0 means the check fires, exit 1 means the check is
/// inert and no number this probe prints may be quoted.
fn run_red_control_arm() -> Result<(), String> {
    match arm_verdict(0) {
        Err(msg) => {
            println!("ARM-CHECK RED CONTROL: refused as designed — {msg}");
            Ok(())
        }
        Ok(_) => Err(
            "ARM-CHECK RED CONTROL FAILED: arm_verdict(0) ACCEPTED. The arm check is inert and \
             every census this binary prints is unverified."
                .to_string(),
        ),
    }
}

fn report(
    grammar: &str,
    input: &str,
    arm_calls: u64,
    before_parse_live: u64,
    before_parse_calls: u64,
) {
    let peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    let snap = SNAP_LIVE.load(Ordering::Relaxed);
    let end_live = LIVE_BYTES.load(Ordering::Relaxed);
    let alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    let alloc_bytes = ALLOC_BYTES.load(Ordering::Relaxed);
    let free_calls = FREE_CALLS.load(Ordering::Relaxed);
    let free_bytes = FREE_BYTES.load(Ordering::Relaxed);
    // ⭐ A FREE CONTINUOUS ORACLE, IN THE `items_agree` SPIRIT: the per-class table
    // and the running total are maintained by two independent code paths, so their
    // sums must agree exactly. A `false` here invalidates the class table outright
    // — and it is also what would catch a TORN snapshot, i.e. a second thread
    // allocating during the copy, which is the assumption every `Relaxed` ordering
    // in this file rests on.
    let live_class_sum: u64 = LIVE_CLASS_BYTES
        .iter()
        .map(|cell| cell.load(Ordering::Relaxed))
        .sum();
    let snap_class_sum: u64 = SNAP_CLASS_BYTES
        .iter()
        .map(|cell| cell.load(Ordering::Relaxed))
        .sum();
    let peak_at_alloc_calls = PEAK_AT_ALLOC_CALLS.load(Ordering::Relaxed);
    let peak_at_free_calls = PEAK_AT_FREE_CALLS.load(Ordering::Relaxed);
    let peak_at_alloc_bytes = PEAK_AT_ALLOC_BYTES.load(Ordering::Relaxed);

    println!(
        "PARSE-ALLOC-CENSUS: grammar={} input={} arm=counting_global_allocator arm_verified_calls={} \
         peak_live={} ({:.2} MB) · snapshot_live={} ({:.2} MB) · snapshot_gap={} \
         · snapshots={} · end_live={} ({:.2} MB) · live_before_parse={} \
         · alloc_calls={} · alloc_bytes={} ({:.2} MB) · free_calls={} · free_bytes={} ({:.2} MB) \
         · parse_alloc_calls={} · churn_ratio={:.4} \
         · peak_at_alloc_calls={} · peak_at_free_calls={} · peak_at_alloc_bytes={} \
         · post_peak_alloc_calls={} · post_peak_alloc_bytes={} · post_peak_free_calls={} \
         · live_class_sum={} · snap_class_sum={} · class_sum_agrees={} · snap_sum_agrees={}",
        grammar,
        input,
        arm_calls,
        peak,
        mb(peak),
        snap,
        mb(snap),
        peak.saturating_sub(snap),
        SNAP_TAKEN.load(Ordering::Relaxed),
        end_live,
        mb(end_live),
        before_parse_live,
        alloc_calls,
        alloc_bytes,
        mb(alloc_bytes),
        free_calls,
        free_bytes,
        mb(free_bytes),
        alloc_calls.saturating_sub(before_parse_calls),
        if peak == 0 { 0.0 } else { alloc_bytes as f64 / peak as f64 },
        peak_at_alloc_calls,
        peak_at_free_calls,
        peak_at_alloc_bytes,
        alloc_calls.saturating_sub(peak_at_alloc_calls),
        alloc_bytes.saturating_sub(peak_at_alloc_bytes),
        free_calls.saturating_sub(peak_at_free_calls),
        live_class_sum,
        snap_class_sum,
        live_class_sum == end_live,
        snap_class_sum == snap,
    );

    // ⭐ THE SIZE-CLASS TABLE IS ONLY ATTRIBUTABLE IF THE CANDIDATE STRUCTURES'
    // ITEM SIZES ARE KNOWN, AND THEY ARE MEASURED HERE RATHER THAN COMPUTED BY
    // HAND FROM THE FIELD LIST. `MemoEntry`'s layout in particular is not
    // hand-derivable — it carries four `Option`s whose niches the compiler is free
    // to pack — and a hand-computed bucket size would have been an inference
    // dressed as a measurement. `memo_bucket`/`memo_fail_bucket` are the `T` a
    // hashbrown table stores per bucket; the table also carries ONE control byte
    // per bucket, which the analysis adds rather than this line hiding.
    println!(
        "PARSE-ALLOC-TYPE-SIZES: ParseNode={} ParseContent={} PgenValue={} shaped_pair={} \
         TapeWord={} MemoEntry={} memo_key={} memo_bucket={} memo_fail_bucket={} \
         String={} node_ref={}",
        std::mem::size_of::<pgen::ast_pipeline::ParseNode<'_>>(),
        std::mem::size_of::<pgen::ast_pipeline::ParseContent<'_>>(),
        std::mem::size_of::<pgen::ast_pipeline::PgenValue<'_>>(),
        std::mem::size_of::<(&str, pgen::ast_pipeline::PgenValue<'_>)>(),
        std::mem::size_of::<pgen::ast_pipeline::TapeWord<'_>>(),
        std::mem::size_of::<pgen::ast_pipeline::MemoEntry<'_>>(),
        std::mem::size_of::<(pgen::ast_pipeline::RuleId, usize)>(),
        std::mem::size_of::<(
            (pgen::ast_pipeline::RuleId, usize),
            pgen::ast_pipeline::MemoEntry<'_>,
        )>(),
        std::mem::size_of::<(pgen::ast_pipeline::RuleId, usize)>(),
        std::mem::size_of::<String>(),
        std::mem::size_of::<&pgen::ast_pipeline::ParseNode<'_>>(),
    );
    println!(
        "PARSE-ALLOC-CLASSES: {} classes (class k = requests in [2^(k-1), 2^k)), \
         live at the snapshot above",
        CLASSES
    );
    for (k, (bytes_cell, count_cell)) in SNAP_CLASS_BYTES
        .iter()
        .zip(SNAP_CLASS_COUNT.iter())
        .enumerate()
    {
        let bytes = bytes_cell.load(Ordering::Relaxed);
        let count = count_cell.load(Ordering::Relaxed);
        if bytes == 0 && count == 0 {
            continue;
        }
        let lo = if k == 0 { 0u64 } else { 1u64 << (k - 1) };
        let hi = 1u64 << k;
        println!(
            "  ALLOC-CLASS class={} lo={} hi={} live_bytes={} live_MB={:.2} live_count={} share={:.6}",
            k,
            lo,
            hi,
            bytes,
            mb(bytes),
            count,
            if snap == 0 { 0.0 } else { bytes as f64 / snap as f64 },
        );
    }
}

#[cfg(feature = "generated_parsers")]
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--red-control-arm") {
        return run_red_control_arm();
    }
    let before_calls = verify_arm()?;
    if args.len() < 3 {
        return Err(format!(
            "Usage: {} <grammar_name> <input_file> [--profile PROFILE] [--hysteresis-bytes N]\n\
             \x20      {0} --red-control-arm",
            args.first().map(String::as_str).unwrap_or("parse_alloc_census_probe")
        ));
    }
    let grammar = args[1].clone();
    let input = args[2].clone();
    let mut profile: Option<String> = None;
    let mut idx = 3;
    while idx < args.len() {
        match args[idx].as_str() {
            "--profile" => {
                profile = Some(
                    args.get(idx + 1)
                        .cloned()
                        .ok_or_else(|| "--profile requires a value".to_string())?,
                );
                idx += 2;
            }
            "--hysteresis-bytes" => {
                let raw = args
                    .get(idx + 1)
                    .ok_or_else(|| "--hysteresis-bytes requires a value".to_string())?;
                let n: u64 = raw
                    .parse()
                    .map_err(|_| format!("--hysteresis-bytes: not an integer: {raw}"))?;
                SNAP_HYSTERESIS.store(n, Ordering::Relaxed);
                idx += 2;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let sample = std::fs::read_to_string(&input)
        .map_err(|e| format!("cannot read input file '{input}': {e}"))?;

    // Everything up to here is setup; the census below still covers the whole
    // process (a peak is a peak), but the pre-parse live figure is reported so a
    // reader can see how much of it the parse is responsible for.
    let before_live = LIVE_BYTES.load(Ordering::Relaxed);
    let before_parse_calls = ALLOC_CALLS.load(Ordering::Relaxed);

    // The SAME entry point `parseability_probe --parse <g> <f> --profile <p>` calls,
    // so this probe measures the parse the tracked ladder measures — and, with
    // `PGEN_REPORT_PARSE_FOOTPRINT=1`, the engine's arena/owned-heap census prints
    // from inside this same process and this same parse.
    let library_options = pgen::parser_registry::LibraryOptions {
        in_dir: None,
        out_dir: None,
    };
    let result = pgen::parser_registry::parse_sample_detail_with_options(
        &grammar,
        &sample,
        profile.as_deref(),
        &library_options,
    );

    let verdict = match result {
        Some(Ok(())) => {
            println!("parse_full passed for grammar '{grammar}' on '{input}'");
            Ok(())
        }
        Some(Err(err)) => Err(format!(
            "parse_full rejected sample for grammar '{grammar}' on '{input}': {err}"
        )),
        None => Err(format!(
            "parseability adapter unavailable for grammar '{grammar}'"
        )),
    };

    // ⛔ The census prints on BOTH verdicts. A rejected parse still allocated, and
    // 428 of the 487 corpus files >= 20 KB REJECT — reporting only on ACCEPT would
    // silently restrict the population to the one that parses standalone.
    report(&grammar, &input, before_calls, before_live, before_parse_calls);

    // `ENGINE-UNIVERSAL-SERVICES.67`(c)(i) — the OCCUPANCY of the dense thin-memo
    // table leg 4 measured at 872 bytes per input byte.
    //
    // ⛔ IT IS READ HERE, AFTER THE PARSE RETURNED, AND THAT IS THE ONLY MOMENT IT
    // WORKS. A live parser holds the table through a `ThinMemoScratchLease`, so
    // during the parse the thread-local is empty; the lease's `Drop` puts it back —
    // which is the same recycling that leaves ~872 B per input byte resident after
    // the call returns. Reading it before the parse, or on another thread, reports
    // `None` and would look exactly like "no census".
    match pgen::ast_pipeline::thin_memo_scratch_stats() {
        Some(stats) => println!(
            "PARSE-THIN-OCCUPANCY: armed={} need_slots={} capacity_slots={} \
             distinct_slots={} store_calls={} clear_calls={} need_bytes={} \
             occupancy={} distinct_exact={}",
            stats.armed,
            stats.need_slots,
            stats.capacity_slots,
            stats.distinct_slots_written,
            stats.store_calls,
            stats.clear_calls,
            stats.need_slots * std::mem::size_of::<u64>(),
            // ⛔ NOT MEASURED, never 0.0000 — a disarmed census and an untouched
            // table are different facts.
            match stats.occupancy() {
                Some(v) => format!("{v:.6}"),
                None => "NOT_MEASURED".to_string(),
            },
            // `distinct_slots` is exact only when nothing was cleared; a cleared
            // slot re-stored is counted twice, so this flag tells the reader
            // whether the number is a count or an upper bound.
            stats.clear_calls == 0,
        ),
        None => println!(
            "PARSE-THIN-OCCUPANCY: NOT AVAILABLE — the thread-local scratch is empty, \
             which means either no generated parser ran on this thread or a lease is \
             still holding it"
        ),
    }
    verdict
}

#[cfg(not(feature = "generated_parsers"))]
fn run() -> Result<(), String> {
    Err("parse_alloc_census_probe requires building with --features generated_parsers".to_string())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Class `k` must hold exactly the requests in `[2^(k-1), 2^k)`. The boundary
    /// cases are what a hand-rolled `leading_zeros` bucket gets wrong, so they are
    /// the ones pinned.
    #[test]
    fn class_of_buckets_by_power_of_two() {
        assert_eq!(class_of(0), 1, "a zero-sized request lands in class 1, not 0");
        assert_eq!(class_of(1), 1);
        assert_eq!(class_of(2), 2);
        assert_eq!(class_of(3), 2);
        assert_eq!(class_of(4), 3);
        assert_eq!(class_of(7), 3);
        assert_eq!(class_of(8), 4);
        assert_eq!(class_of(1 << 20), 21);
        assert_eq!(class_of((1 << 20) - 1), 20);
        assert!(class_of(usize::MAX) < CLASSES, "every request fits the table");
    }

    #[test]
    fn arm_verdict_refuses_a_dead_allocator() {
        assert!(arm_verdict(0).is_err(), "0 allocations means the arm is not installed");
        assert_eq!(arm_verdict(1).ok(), Some(1));
    }
}
