//! `ENGINE-UNIVERSAL-SERVICES.67`(b) leg 3 — the ORACLE for the `typed_arena`
//! chunk-capacity model.
//!
//! # What it is for
//!
//! [`pgen::ast_pipeline::ArenaChunkModel`] reproduces `typed-arena` 2.0.2's
//! chunk arithmetic so a parse can report the bytes its arenas actually took
//! from the allocator — the exclusion that kept `.67`(b)'s arena share stuck in
//! a 28–71 % band. A model of somebody else's algorithm is a CLAIM about that
//! algorithm, and `-0125`'s carry-forward is that re-running an instrument is
//! not verification: *"take each published sentence, name the measurement that
//! would refute it, and check that the measurement was actually taken."*
//!
//! The sentence here is **"the model's `total_bytes()` equals what the arena
//! took from the allocator."** The measurement that refutes it is this probe: a
//! counting `GlobalAlloc` (the shape `regex_alloc_census_probe` already uses)
//! wrapped around a REAL `typed_arena::Arena`, driven by the same call sequence
//! as the model, compared for EXACT equality. Not a tolerance — an equality.
//!
//! # Why the comparison is sound
//!
//! Inside a measured window the only live allocations are the arena's own: the
//! item type is a POD array that owns no heap, the operation script is built
//! before the window opens, and `alloc_extend` is fed an allocation-free
//! `Map<Range>`. So `live_bytes(after) − live_bytes(before)` is exactly the
//! arena's chunk storage plus its `ChunkList::rest` spine — which is what
//! `total_bytes()` claims to be.
//!
//! # Usage
//!
//! ```text
//! cargo build --release --bin arena_chunk_oracle_probe
//! ./target/release/arena_chunk_oracle_probe                # exit 0 iff every case matches
//! ./target/release/arena_chunk_oracle_probe --red-control  # exit 0 iff the check FAILS
//! ./target/release/arena_chunk_oracle_probe --verbose
//! ./target/release/arena_chunk_oracle_probe --residency 22020075  # see `residency`
//! ```
//!
//! ⛔ `--red-control` exists because a comparison that has never been observed
//! failing is not evidence. It perturbs the MODEL by one chunk-worth of bytes
//! and requires the probe to notice; if the perturbed run still passes, the
//! probe is measuring nothing and exits non-zero for that reason instead.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use pgen::ast_pipeline::ArenaChunkModel;

static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static FREE_BYTES: AtomicU64 = AtomicU64::new(0);

/// System-backed counting allocator. `realloc` is one alloc of the new size and
/// one free of the old, so the live-byte difference stays exact across a `Vec`
/// growth — which is precisely how `ChunkList::rest` grows.
struct CountingAlloc;

// SAFETY: every operation is delegated verbatim to `System`; the counters are
// side-effect-only relaxed atomics.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        FREE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        FREE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn live_bytes() -> i64 {
    ALLOC_BYTES.load(Ordering::Relaxed) as i64 - FREE_BYTES.load(Ordering::Relaxed) as i64
}

/// A POD item that owns no heap, so every byte the window sees is the arena's.
#[derive(Clone, Copy)]
struct Pod<const N: usize>([u8; N]);

impl<const N: usize> Pod<N> {
    /// `[u8; N]` has no `Default` for a const-generic `N`, and a zeroed array
    /// is what this probe wants anyway: the VALUES never matter, only the bytes
    /// the arena reserves to hold them.
    const fn zero() -> Self {
        Pod([0u8; N])
    }
}

/// One step of a scripted arena workload.
#[derive(Clone, Copy, Debug)]
enum Op {
    /// `Arena::alloc`.
    Alloc,
    /// `Arena::alloc_extend` with an exact-size iterator of `n` items.
    Extend(usize),
}

/// Drive a real arena and the model through the same script; return
/// (measured live bytes, model bytes, model item count, arena item count).
fn run_case<const N: usize>(script: &[Op]) -> (i64, usize, usize, usize) {
    let before = live_bytes();
    let arena: typed_arena::Arena<Pod<N>> = typed_arena::Arena::new();
    let mut model = ArenaChunkModel::new(std::mem::size_of::<Pod<N>>());
    for op in script {
        match *op {
            Op::Alloc => {
                arena.alloc(Pod::<N>::zero());
                model.alloc();
            }
            Op::Extend(n) => {
                let iter = (0..n).map(|_| Pod::<N>::zero());
                let hint = iter.size_hint().0;
                let produced = arena.alloc_extend(iter).len();
                model.alloc_extend(hint, produced);
            }
        }
    }
    let measured = live_bytes() - before;
    let arena_items = arena.len();
    let out = (measured, model.total_bytes(), model.len_items(), arena_items);
    drop(arena);
    out
}

/// A deterministic LCG — the sequences must be reproducible, and pulling in a
/// PRNG crate for eight lines would add a dependency to a measurement-only bin.
struct Lcg(u64);

impl Lcg {
    fn next_in(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % bound.max(1)
    }
}

struct Case {
    name: String,
    item_bytes: usize,
    measured: i64,
    modelled: usize,
    model_items: usize,
    arena_items: usize,
}

impl Case {
    fn bytes_agree(&self, perturb: usize) -> bool {
        self.measured == (self.modelled + perturb) as i64
    }
    fn items_agree(&self) -> bool {
        self.model_items == self.arena_items
    }
}

fn make_case<const N: usize>(name: &str, script: &[Op]) -> Case {
    let (measured, modelled, model_items, arena_items) = run_case::<N>(script);
    Case {
        name: format!("{name} @ {N}B"),
        item_bytes: N,
        measured,
        modelled,
        model_items,
        arena_items,
    }
}

/// `run_case` is generic over the item size, so the sizes have to be named at
/// compile time. These bracket the four real arenas: `ParseNode` is 48 bytes,
/// `PgenValue` and the `(&str, PgenValue)` pair sit in the 32–64 band, `String`
/// is 24, and 1 / 1536 are the degenerate ends (a 1024-item initial chunk, and
/// an item larger than `INITIAL_SIZE` so the initial capacity clamps to 1).
fn push_all_sizes(cases: &mut Vec<Case>, name: &str, script: &[Op]) {
    cases.push(make_case::<1>(name, script));
    cases.push(make_case::<24>(name, script));
    cases.push(make_case::<48>(name, script));
    cases.push(make_case::<64>(name, script));
    cases.push(make_case::<1536>(name, script));
}

/// `--residency N` — allocate `N` 48-byte items into a REAL arena and report
/// what the model says was ALLOCATED versus what it says was WRITTEN, then hold
/// the arena alive across the print.
///
/// ⛔ **THIS ARM EXISTS BECAUSE THE OTHER ONE CANNOT ANSWER THE QUESTION THE LEAF
/// IS ASKING.** The byte-equality arm proves the model reproduces what the
/// ALLOCATOR handed out; `.67`(b) is attributing PEAK RSS, and on any
/// lazy-commit OS a freshly allocated chunk is untouched anonymous memory whose
/// pages enter RSS only when written. So "allocated" is an upper bound on
/// residency, and the gap is exactly the arena's unused chunk slack.
///
/// The experiment is designed to be decisive rather than calibrated: run it at
/// `N` and at `N + 1` where `N` fills the last chunk exactly. One extra item
/// DOUBLES the allocated bytes and leaves the written bytes unchanged, so the
/// two peak-RSS readings separate the two hypotheses with no floor subtraction
/// and no model of the process floor at all. Driver:
/// `docs/tasks/artifacts/eus67/measure_chunk_residency.sh`.
fn residency(n: usize) {
    let arena: typed_arena::Arena<Pod<48>> = typed_arena::Arena::new();
    let mut model = ArenaChunkModel::new(std::mem::size_of::<Pod<48>>());
    for _ in 0..n {
        arena.alloc(Pod::<48>::zero());
        model.alloc();
    }
    let mb = |b: usize| b as f64 / (1024.0 * 1024.0);
    println!(
        "ARENA-CHUNK-ORACLE: residency n={} arena_len={} chunks={} allocated={:.2} MB \
         touched={:.2} MB slack={:.2} MB",
        n,
        arena.len(),
        model.chunks(),
        mb(model.total_bytes()),
        mb(model.touched_total_bytes()),
        mb(model.total_bytes() - model.touched_total_bytes()),
    );
    // Hold the arena until after the print so the peak the caller measures
    // covers it. `black_box` is not available outside nightly here; reading
    // `len()` again is enough to keep it alive and is not optimised away
    // because it is printed.
    if arena.len() != n {
        std::process::exit(3);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let red_control = args.iter().any(|a| a == "--red-control");
    let verbose = args.iter().any(|a| a == "--verbose");

    if let Some(i) = args.iter().position(|a| a == "--residency") {
        match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
            Some(n) => {
                residency(n);
                return;
            }
            None => {
                eprintln!("ARENA-CHUNK-ORACLE: --residency needs an item count");
                std::process::exit(2);
            }
        }
    }

    let mut scripts: Vec<(String, Vec<Op>)> = Vec::new();

    // (1) Pure `alloc`, straddling every chunk boundary a 48-byte item hits.
    for n in [0usize, 1, 20, 21, 22, 62, 63, 64, 1_000, 100_000] {
        scripts.push((format!("alloc x{n}"), vec![Op::Alloc; n]));
    }
    // ⭐ (2) The SPINE-FREE case: one chunk, nothing retired, so `rest` never
    // allocates and the comparison isolates the CHUNK model from the `std`-`Vec`
    // spine model. Without it, a compensating pair of errors would pass.
    scripts.push(("alloc x1 (single chunk, no spine)".into(), vec![Op::Alloc]));

    // (3) Pure `alloc_extend` — the arm `-0125` showed no closed form covers.
    for n in [1usize, 7, 21, 22, 100, 1_000, 4_096, 50_000] {
        scripts.push((format!("extend({n})"), vec![Op::Extend(n)]));
    }
    // (4) Extend sequences with the SAME total and different shapes: the direct
    // executable form of "capacity depends on the sequence, not the total".
    scripts.push((
        "extend 1000x1".into(),
        (0..1_000).map(|_| Op::Extend(1)).collect(),
    ));
    scripts.push((
        "extend 10x100".into(),
        (0..10).map(|_| Op::Extend(100)).collect(),
    ));
    scripts.push(("extend 1x1000".into(), vec![Op::Extend(1_000)]));

    // (5) Mixed pseudo-random scripts — the shape a real parse produces, where
    // `alloc` and `alloc_extend` interleave against a partly filled chunk.
    for seed in 0..24u64 {
        let mut rng = Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1));
        let len = 200 + rng.next_in(1_800);
        let script: Vec<Op> = (0..len)
            .map(|_| {
                if rng.next_in(3) == 0 {
                    Op::Extend(rng.next_in(64))
                } else {
                    Op::Alloc
                }
            })
            .collect();
        scripts.push((format!("mixed seed {seed} ({len} ops)"), script));
    }

    let mut cases: Vec<Case> = Vec::new();
    for (name, script) in &scripts {
        push_all_sizes(&mut cases, name, script);
    }

    // ⛔ The perturbation is applied to the MODEL side only, and it is one
    // 48-byte item — the smallest error this leg could plausibly make.
    let perturb = if red_control { 48 } else { 0 };

    let mut byte_failures = 0usize;
    let mut item_failures = 0usize;
    for case in &cases {
        let ok_bytes = case.bytes_agree(perturb);
        let ok_items = case.items_agree();
        if !ok_bytes {
            byte_failures += 1;
        }
        if !ok_items {
            item_failures += 1;
        }
        if verbose || !ok_bytes || !ok_items {
            println!(
                "{:<48} item={:>5}B measured={:>10} modelled={:>10} items {}/{} {}{}",
                case.name,
                case.item_bytes,
                case.measured,
                case.modelled + perturb,
                case.model_items,
                case.arena_items,
                if ok_bytes { "BYTES-OK" } else { "BYTES-MISMATCH" },
                if ok_items { "" } else { " ITEMS-MISMATCH" },
            );
        }
    }

    println!(
        "\nARENA-CHUNK-ORACLE: {} case(s) over {} script(s) x 5 item sizes: \
         {} byte mismatch(es), {} item mismatch(es)",
        cases.len(),
        scripts.len(),
        byte_failures,
        item_failures
    );

    if red_control {
        // The control passes when the perturbed model is REJECTED. A control
        // that goes green is the failure being reported.
        if byte_failures == 0 {
            println!(
                "ARENA-CHUNK-ORACLE: RED CONTROL FAILED — a model perturbed by 48 bytes was \
                 still accepted, so the comparison is measuring nothing"
            );
            std::process::exit(1);
        }
        println!(
            "ARENA-CHUNK-ORACLE: RED CONTROL OK — {byte_failures} of {} case(s) rejected the \
             48-byte perturbation",
            cases.len()
        );
        return;
    }

    if byte_failures == 0 && item_failures == 0 {
        println!(
            "ARENA-CHUNK-ORACLE: PASS — the chunk model reproduces typed_arena 2.0.2's resident \
             bytes EXACTLY on every case"
        );
    } else {
        println!(
            "ARENA-CHUNK-ORACLE: FAIL — the chunk model does not reproduce typed_arena's \
             resident bytes"
        );
        std::process::exit(1);
    }
}
