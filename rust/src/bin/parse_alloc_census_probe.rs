//! Parse allocation-census probe — the ALLOCATOR-SIDE leg of the parse-footprint
//! attribution (`ENGINE-UNIVERSAL-SERVICES.67`(b) LEG 4) and the LIFETIME tape that
//! settles the memo rehash spike (`.67`(c)(iii)).
//!
//! TWO INSTRUMENTS, AND THE SECOND EXISTS BECAUSE THE FIRST WAS READ FOR SOMETHING IT
//! CANNOT SAY. The size-class SNAPSHOT answers *what is live at one instant*, as a
//! TOTAL PER CLASS. Leg 4 read it as *"two success tables live at once ⇒ that is the
//! rehash window"* — which is a claim about SUCCESSION, and a snapshot has no
//! succession in it. Worse, naming individual structures inside a class total is a
//! SUBSET-SUM search, and a total has many decompositions: on `floor.sv` that search
//! named FOUR `T=296` tables (512, 128, 64, 64 buckets) and the lifetime tape shows
//! the process never made an allocation of 19 016 or 38 024 bytes AT ALL, anywhere on
//! the ladder — `floor.sv` has exactly ONE table-shaped allocation. So the tape records
//! LIFETIMES — see [`tape_note_alloc`] — one row per allocation, and both succession
//! and identity become observations instead of inferences.
//!
//! ⛔ SCOPE, STATED HERE SO IT IS NOT OVERREAD: on the EIGHT attributed body inputs the
//! subset-sum naming and the tape AGREE table for table. The refutation is the FLOOR
//! row, which leg 4 had already excluded from every band as "the floor, not a shape",
//! and where it had already recorded the search "observed fabricating".
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
///
/// ⛔⛔ **THAT SENTENCE IS A REASONED BOUND, NOT A GUARANTEED ONE, AND [`class_of`]
/// USED TO TRUST IT.** A `Layout` may carry any size up to `isize::MAX`, so a
/// request of 2^47 bytes or more is REPRESENTABLE even though no host can serve it —
/// and the unclamped `class_of` returned up to 64, indexing a 48-element array from
/// inside the global allocator. Not undefined behaviour (Rust bounds-checks) but a
/// PANIC in `alloc`, which is worse than the impossible allocation failing on its
/// own. Found by RUNNING the probe's own unit test for the first time: the
/// assertion `class_of(usize::MAX) < CLASSES` had been in the file since `-0128`
/// and had never executed (`-0129` could run the library tests, not this bin's).
/// ⇒ [`class_of`] now saturates, and the test pins the saturation rather than the
/// false claim. **A test that has never been run is not a test.**
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

/// ⭐⭐ **THE SNAPSHOT'S OWN ORDINALS — `ENGINE-UNIVERSAL-SERVICES.67`(c)(iii).**
/// The size-class snapshot is taken at the HYSTERESIS instant, which is within one
/// hysteresis step of the peak and **is not the same instant**. `-0128` reasoned
/// about "two tables live at once" from the snapshot and stated it about the peak;
/// the two can differ, so the snapshot's position in the event stream is recorded
/// and the tape control below compares like with like instead of assuming they
/// coincide.
static SNAP_AT_ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static SNAP_AT_FREE_CALLS: AtomicU64 = AtomicU64::new(0);

/// ⛔⛔⛔ **ONE CLOCK, AND IT EXISTS BECAUSE THE FIRST CUT HAD TWO.** `ALLOC_CALLS`
/// and `FREE_CALLS` are separate counters, so an allocation stamped with alloc
/// ordinal 988 and a release stamped with free ordinal 195 are not out of order —
/// they are on different clocks and simply cannot be compared. "Was A still live
/// when B was allocated" is a comparison ACROSS the two, and answering it from the
/// pair silently produces a confident wrong verdict.
///
/// ⭐ The tape's own CONTROL could not have caught it: "live at instant I" compares
/// alloc ordinals to `I`'s alloc ordinal and free ordinals to `I`'s free ordinal —
/// each comparison stays inside one clock, so the control passes while succession is
/// nonsense. **A control validates the property it is built from.** Succession is a
/// different property and needed its own instrument: a single counter every
/// allocation AND every release advances.
static EVENT_SEQ: AtomicU64 = AtomicU64::new(0);
static PEAK_AT_EVENT_SEQ: AtomicU64 = AtomicU64::new(0);
static SNAP_AT_EVENT_SEQ: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// `ENGINE-UNIVERSAL-SERVICES.67`(c)(iii) — THE LARGE-ALLOCATION LIFETIME TAPE.
//
// ⛔ WHY A SECOND INSTRUMENT AND NOT MORE READING OF THE FIRST. `-0128` published
// *"two success tables are live at once … that is the REHASH window"* from the
// size-class SNAPSHOT. A snapshot cannot say that: it shows co-residency, not
// succession, and it is taken at the hysteresis instant rather than at the peak.
// `floor.sv` falsifies the identification outright — it holds FOUR `T=296` tables
// at once, two of them the SAME size, which no rehash can produce. Succession is a
// LIFETIME property, so it needs a lifetime instrument.
//
// WHAT IT RECORDS. For every allocation at or above [`TAPE_MIN_BYTES`]: its
// requested size, the `ALLOC_CALLS` ordinal at which it was made, and the
// `FREE_CALLS` ordinal at which it was released ([`TAPE_NEVER_FREED`] if it never
// was). Those three make "was X live at the instant the peak was set" EXACTLY
// decidable against the `(peak_at_alloc_calls, peak_at_free_calls)` pair the probe
// already records — and they make "was X superseded by Y" decidable too.
//
// ⛔ IT MUST NOT ALLOCATE. Everything here is static storage and atomics; a
// `#[global_allocator]` that allocates re-enters itself.
// ---------------------------------------------------------------------------

/// Entries the tape can hold: 2^20, in 3 × 8 MB of static storage plus a 32 MB index.
///
/// ⛔ SIZED BY MEASUREMENT, NOT BY GUESS, AND THE FIRST GUESS WAS WRONG. 2^16 looked
/// generous — most ladder inputs record 59 to 214 entries at an 8 KiB threshold — and
/// `br_gh330.v`, a **36 KB** file, made **251 438** of them: 65 536 recorded (the cap)
/// and **185 902 overflowed**. ⛔⛔ AND THE TRUNCATED TAPE WAS NOT A SAMPLE OF THE
/// POPULATION — it was a PREFIX of it, and reading it as one is wrong by a factor of
/// three: the truncated table showed 65 443 allocations of exactly 8 192 bytes, and the
/// complete run shows **220 773**, plus **30 561** of exactly 16 384, essentially all
/// freed, mean lifetime **29.4 events**. A tape that had silently truncated there would
/// have produced a complete-looking lifetime table for the input whose allocation
/// behaviour is the most unusual on the ladder, and a population figure three times too
/// small. It did not, because overflow is COUNTED and `complete=` is a word in the
/// output the analysers refuse on.
const TAPE_CAP: usize = 1 << 20;
/// Open-addressed pointer index, load factor ≤ 0.5 by construction of [`TAPE_CAP`].
const TAPE_IDX_CAP: usize = TAPE_CAP << 1;
/// `free_seq` of an allocation the process never released.
const TAPE_NEVER_FREED: u64 = u64::MAX;
/// Index slot states. `0` terminates a probe chain; `u64::MAX` is a tombstone a
/// probe must skip past, so a pointer freed and re-issued cannot shadow its
/// successor.
const TAPE_IDX_EMPTY: u64 = 0;
const TAPE_IDX_TOMB: u64 = u64::MAX;

static TAPE_SIZE: [AtomicU64; TAPE_CAP] = [ZERO; TAPE_CAP];
static TAPE_ALLOC_SEQ: [AtomicU64; TAPE_CAP] = [ZERO; TAPE_CAP];
static TAPE_FREE_SEQ: [AtomicU64; TAPE_CAP] = [ZERO; TAPE_CAP];
static TAPE_IDX_PTR: [AtomicU64; TAPE_IDX_CAP] = [ZERO; TAPE_IDX_CAP];
static TAPE_IDX_SLOT: [AtomicU64; TAPE_IDX_CAP] = [ZERO; TAPE_IDX_CAP];

static TAPE_LEN: AtomicU64 = AtomicU64::new(0);
/// Allocations at or above the threshold that did not fit the tape. ⛔ A non-zero
/// value INVALIDATES every lifetime conclusion and the report says so rather than
/// printing a partial table that looks whole.
static TAPE_OVERFLOW: AtomicU64 = AtomicU64::new(0);
/// Frees at or above the threshold with no live tape entry for the pointer. Expected
/// to be zero; reported because a non-zero value means the index lost an entry and
/// the free ordinals cannot be trusted.
static TAPE_UNMATCHED_FREES: AtomicU64 = AtomicU64::new(0);
/// Longest probe chain walked, so a degenerate index shows up as a number rather
/// than as a slowdown nobody attributes.
static TAPE_PROBE_MAX: AtomicU64 = AtomicU64::new(0);
static TAPE_MIN_BYTES: AtomicU64 = AtomicU64::new(8192);
/// Set before the report prints. The report itself allocates (formatting), and a
/// tape that kept recording during its own printing would describe a different run
/// on every line.
static TAPE_FROZEN: AtomicU64 = AtomicU64::new(0);

#[inline]
fn tape_hash(ptr: u64) -> usize {
    // Fibonacci hashing on the pointer with its 16-byte malloc alignment shifted
    // out; the low bits of a system-allocator pointer are constant and would put
    // every entry in one chain.
    let mixed = (ptr >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (mixed >> (64 - TAPE_IDX_CAP.trailing_zeros())) as usize & (TAPE_IDX_CAP - 1)
}

/// Record an allocation of `size` made at event `alloc_seq`. Ignores anything below the
/// threshold and anything after the tape is frozen.
#[inline]
fn tape_note_alloc(ptr: *mut u8, size: usize, alloc_seq: u64) {
    if ptr.is_null()
        || (size as u64) < TAPE_MIN_BYTES.load(Ordering::Relaxed)
        || TAPE_FROZEN.load(Ordering::Relaxed) != 0
    {
        return;
    }
    let slot = TAPE_LEN.fetch_add(1, Ordering::Relaxed);
    if slot >= TAPE_CAP as u64 {
        // Leave TAPE_LEN past the cap; the report reads it and refuses.
        TAPE_OVERFLOW.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let slot = slot as usize;
    TAPE_SIZE[slot].store(size as u64, Ordering::Relaxed);
    TAPE_ALLOC_SEQ[slot].store(alloc_seq, Ordering::Relaxed);
    TAPE_FREE_SEQ[slot].store(TAPE_NEVER_FREED, Ordering::Relaxed);

    let key = ptr as u64;
    let mut idx = tape_hash(key);
    let mut probes = 0u64;
    loop {
        let cur = TAPE_IDX_PTR[idx].load(Ordering::Relaxed);
        if cur == TAPE_IDX_EMPTY || cur == TAPE_IDX_TOMB {
            TAPE_IDX_PTR[idx].store(key, Ordering::Relaxed);
            TAPE_IDX_SLOT[idx].store(slot as u64, Ordering::Relaxed);
            break;
        }
        idx = (idx + 1) & (TAPE_IDX_CAP - 1);
        probes += 1;
        if probes >= TAPE_IDX_CAP as u64 {
            // Cannot happen while TAPE_LEN <= TAPE_CAP = TAPE_IDX_CAP / 2, but a
            // full table must be a reported number and not an infinite loop.
            TAPE_OVERFLOW.fetch_add(1, Ordering::Relaxed);
            break;
        }
    }
    if probes > TAPE_PROBE_MAX.load(Ordering::Relaxed) {
        TAPE_PROBE_MAX.store(probes, Ordering::Relaxed);
    }
}

/// Stamp the event seq at which the live tape entry for `ptr` was released.
#[inline]
fn tape_note_free(ptr: *mut u8, size: usize, free_seq: u64) {
    if ptr.is_null()
        || (size as u64) < TAPE_MIN_BYTES.load(Ordering::Relaxed)
        || TAPE_FROZEN.load(Ordering::Relaxed) != 0
    {
        return;
    }
    let key = ptr as u64;
    let mut idx = tape_hash(key);
    let mut probes = 0u64;
    loop {
        let cur = TAPE_IDX_PTR[idx].load(Ordering::Relaxed);
        if cur == TAPE_IDX_EMPTY {
            TAPE_UNMATCHED_FREES.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if cur == key {
            let slot = TAPE_IDX_SLOT[idx].load(Ordering::Relaxed) as usize;
            if slot < TAPE_CAP && TAPE_FREE_SEQ[slot].load(Ordering::Relaxed) == TAPE_NEVER_FREED {
                TAPE_FREE_SEQ[slot].store(free_seq, Ordering::Relaxed);
                TAPE_IDX_PTR[idx].store(TAPE_IDX_TOMB, Ordering::Relaxed);
                return;
            }
        }
        idx = (idx + 1) & (TAPE_IDX_CAP - 1);
        probes += 1;
        if probes >= TAPE_IDX_CAP as u64 {
            TAPE_UNMATCHED_FREES.fetch_add(1, Ordering::Relaxed);
            return;
        }
    }
}

#[inline]
fn class_of(size: usize) -> usize {
    // `size.max(1)` so a zero-sized request lands in class 1 rather than 0; class 0
    // then means "never used", which keeps the printed table honest.
    //
    // ⛔ The `.min(CLASSES - 1)` is the out-of-bounds guard described on [`CLASSES`].
    // It cannot change any measurement on this ladder — the largest allocation seen
    // is ~1.08 GB, class 31 — because it only binds at 2^47 bytes and above.
    ((usize::BITS - size.max(1).leading_zeros()) as usize).min(CLASSES - 1)
}

struct CountingAlloc;

// SAFETY: every operation delegates verbatim to `System`; the counters are
// side-effect-only atomics and the snapshot writes touch only static storage, so
// nothing here can allocate and re-enter.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let seq = note_alloc(layout.size());
        let ptr = unsafe { System.alloc(layout) };
        tape_note_alloc(ptr, layout.size(), seq);
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let seq = note_alloc(layout.size());
        let ptr = unsafe { System.alloc_zeroed(layout) };
        tape_note_alloc(ptr, layout.size(), seq);
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A `realloc` is one free of the old size plus one alloc of the new, in
        // that order: the growth of a `Vec` must not be charged as if both buffers
        // were live at once, because they are not.
        let free_seq = note_free(layout.size());
        tape_note_free(ptr, layout.size(), free_seq);
        let seq = note_alloc(new_size);
        let out = unsafe { System.realloc(ptr, layout, new_size) };
        tape_note_alloc(out, new_size, seq);
        out
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let free_seq = note_free(layout.size());
        tape_note_free(ptr, layout.size(), free_seq);
        unsafe { System.dealloc(ptr, layout) }
    }
}

/// Returns the EVENT sequence number this allocation was given — one clock shared
/// with [`note_free`], so "was A still live when B was allocated" is a comparison
/// between two readings of the SAME counter. It also stores the per-kind ordinals the
/// rest of the census reports.
#[inline]
fn note_alloc(size: usize) -> u64 {
    let k = class_of(size);
    let ord = ALLOC_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
    let seq = EVENT_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    ALLOC_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_BYTES[k].fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_COUNT[k].fetch_add(1, Ordering::Relaxed);
    let live = LIVE_BYTES.fetch_add(size as u64, Ordering::Relaxed) + size as u64;
    if live > PEAK_LIVE_BYTES.load(Ordering::Relaxed) {
        PEAK_LIVE_BYTES.store(live, Ordering::Relaxed);
        PEAK_AT_ALLOC_CALLS.store(ord, Ordering::Relaxed);
        PEAK_AT_FREE_CALLS.store(FREE_CALLS.load(Ordering::Relaxed), Ordering::Relaxed);
        PEAK_AT_EVENT_SEQ.store(seq, Ordering::Relaxed);
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
            SNAP_AT_ALLOC_CALLS.store(ord, Ordering::Relaxed);
            SNAP_AT_FREE_CALLS.store(FREE_CALLS.load(Ordering::Relaxed), Ordering::Relaxed);
            SNAP_AT_EVENT_SEQ.store(seq, Ordering::Relaxed);
            SNAP_TAKEN.fetch_add(1, Ordering::Relaxed);
        }
    }
    seq
}

/// Returns the EVENT sequence number this release was given, on the same clock
/// [`note_alloc`] stamps allocations with.
#[inline]
fn note_free(size: usize) -> u64 {
    let k = class_of(size);
    FREE_CALLS.fetch_add(1, Ordering::Relaxed);
    let seq = EVENT_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    FREE_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    LIVE_CLASS_BYTES[k].fetch_sub(size as u64, Ordering::Relaxed);
    LIVE_CLASS_COUNT[k].fetch_sub(1, Ordering::Relaxed);
    LIVE_BYTES.fetch_sub(size as u64, Ordering::Relaxed);
    seq
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
    // ⛔ FREEZE FIRST. Printing allocates, so a tape still recording during its own
    // report would describe a different run on every line — and the frees the
    // report's own temporaries perform would stamp free ordinals onto entries the
    // parse left live. Frozen here, before a single figure is read.
    TAPE_FROZEN.store(1, Ordering::Relaxed);
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

    // `ENGINE-UNIVERSAL-SERVICES.67`(c)(iii) — the LIFETIME tape. Every allocation
    // at or above the threshold, with the ordinals that make "live at the peak" and
    // "superseded by" decidable rather than inferred from a co-residency snapshot.
    let recorded = TAPE_LEN.load(Ordering::Relaxed);
    let overflow = TAPE_OVERFLOW.load(Ordering::Relaxed);
    println!(
        "PARSE-ALLOC-TAPE: min_bytes={} capacity={} recorded={} overflow={} \
         unmatched_frees={} probe_max={} complete={} \
         peak_at_event_seq={} snap_at_event_seq={} \
         peak_at_alloc_calls={} peak_at_free_calls={} \
         snap_at_alloc_calls={} snap_at_free_calls={} never_freed_sentinel={}",
        TAPE_MIN_BYTES.load(Ordering::Relaxed),
        TAPE_CAP,
        recorded.min(TAPE_CAP as u64),
        overflow,
        TAPE_UNMATCHED_FREES.load(Ordering::Relaxed),
        TAPE_PROBE_MAX.load(Ordering::Relaxed),
        // ⛔ ONE WORD THE ANALYSIS KEYS ON. A truncated lifetime table cannot be
        // told from a complete one by looking at it, so the probe says which it is
        // and the analyser refuses an incomplete one rather than averaging it.
        overflow == 0 && TAPE_UNMATCHED_FREES.load(Ordering::Relaxed) == 0,
        PEAK_AT_EVENT_SEQ.load(Ordering::Relaxed),
        SNAP_AT_EVENT_SEQ.load(Ordering::Relaxed),
        peak_at_alloc_calls,
        peak_at_free_calls,
        SNAP_AT_ALLOC_CALLS.load(Ordering::Relaxed),
        SNAP_AT_FREE_CALLS.load(Ordering::Relaxed),
        TAPE_NEVER_FREED,
    );
    for slot in 0..(recorded.min(TAPE_CAP as u64) as usize) {
        println!(
            "  ALLOC-TAPE slot={} size={} alloc_seq={} free_seq={}",
            slot,
            TAPE_SIZE[slot].load(Ordering::Relaxed),
            TAPE_ALLOC_SEQ[slot].load(Ordering::Relaxed),
            TAPE_FREE_SEQ[slot].load(Ordering::Relaxed),
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
            "Usage: {} <grammar_name> <input_file> [--profile PROFILE] [--hysteresis-bytes N] \
             [--tape-min-bytes N]\n\
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
            "--tape-min-bytes" => {
                let raw = args
                    .get(idx + 1)
                    .ok_or_else(|| "--tape-min-bytes requires a value".to_string())?;
                let n: u64 = raw
                    .parse()
                    .map_err(|_| format!("--tape-min-bytes: not an integer: {raw}"))?;
                TAPE_MIN_BYTES.store(n, Ordering::Relaxed);
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
        // ⛔ SATURATION, not "it fits". A `Layout` may carry any size up to
        // `isize::MAX`; the unclamped form returned 64 for this input and indexed a
        // 48-element array from inside the global allocator. The clamp binds only at
        // 2^47 bytes and above, so no measured class on any real allocation moves.
        assert_eq!(
            class_of(usize::MAX),
            CLASSES - 1,
            "an impossible-but-representable size must SATURATE, never index past the table"
        );
        assert_eq!(class_of(1 << 46), 47, "the last class reachable without clamping");
        assert_eq!(class_of(1 << 47), CLASSES - 1, "the first clamped size");
    }

    #[test]
    fn arm_verdict_refuses_a_dead_allocator() {
        assert!(arm_verdict(0).is_err(), "0 allocations means the arm is not installed");
        assert_eq!(arm_verdict(1).ok(), Some(1));
    }

    /// A pointer index that can walk off its own array is a use-after-bounds, and
    /// the shift/mask pair that produces it is exactly the kind of expression that
    /// is right for the sizes you tried and wrong for the ones you did not.
    #[test]
    fn tape_hash_stays_inside_the_index() {
        for raw in [
            1u64,
            16,
            0x1_0000,
            0x7fff_ffff_ffff_fff0,
            u64::MAX & !0xf,
            0x0000_7f9c_1234_5670,
        ] {
            assert!(
                tape_hash(raw) < TAPE_IDX_CAP,
                "tape_hash({raw:#x}) escaped the index"
            );
        }
    }

    /// The three sentinel values must be mutually distinguishable, or a live entry
    /// reads as a tombstone (or worse, a freed one reads as live and the whole
    /// lifetime column is silently wrong in the flattering direction).
    #[test]
    fn tape_sentinels_are_distinct() {
        assert_ne!(TAPE_IDX_EMPTY, TAPE_IDX_TOMB);
        assert_ne!(TAPE_NEVER_FREED, 0);
        assert_eq!(TAPE_NEVER_FREED, TAPE_IDX_TOMB, "documented: both are u64::MAX");
        assert_eq!(TAPE_IDX_CAP, TAPE_CAP * 2, "load factor <= 0.5 by construction");
    }

    /// ⭐ THE ONE THAT MATTERS: an allocation is recorded with `NEVER_FREED` and the
    /// matching free STAMPS it. If this stamping is broken every allocation looks
    /// immortal, which is precisely the finding this instrument exists to test —
    /// so a false positive here would confirm `-0128`'s "never freed" result by
    /// construction rather than by measurement.
    ///
    /// The sizes are fictional and enormous so no allocation the test binary itself
    /// makes can be confused for them; nothing is ever allocated, only recorded.
    #[test]
    fn tape_stamps_a_free_and_ignores_what_is_below_threshold() {
        let live_marker: usize = 0xC0FF_EE00;
        let freed_marker: usize = 0xC0FF_EE01;
        let below_marker: usize = 7; // below the 8 KiB default threshold
        let live_ptr = 0x1_0000_0000_usize as *mut u8;
        let freed_ptr = 0x2_0000_0000_usize as *mut u8;
        let small_ptr = 0x3_0000_0000_usize as *mut u8;

        tape_note_alloc(live_ptr, live_marker, 11);
        tape_note_alloc(freed_ptr, freed_marker, 22);
        tape_note_alloc(small_ptr, below_marker, 33);
        tape_note_free(freed_ptr, freed_marker, 44);
        tape_note_free(small_ptr, below_marker, 55);

        let find = |size: u64| -> Option<(u64, u64)> {
            (0..TAPE_LEN.load(Ordering::Relaxed).min(TAPE_CAP as u64) as usize)
                .find(|&slot| TAPE_SIZE[slot].load(Ordering::Relaxed) == size)
                .map(|slot| {
                    (
                        TAPE_ALLOC_SEQ[slot].load(Ordering::Relaxed),
                        TAPE_FREE_SEQ[slot].load(Ordering::Relaxed),
                    )
                })
        };

        assert_eq!(
            find(live_marker as u64),
            Some((11, TAPE_NEVER_FREED)),
            "an allocation never freed must still read NEVER_FREED"
        );
        assert_eq!(
            find(freed_marker as u64),
            Some((22, 44)),
            "the free ordinal must be stamped onto the entry the pointer indexes"
        );
        assert_eq!(
            find(below_marker as u64),
            None,
            "an allocation below the threshold is not recorded at all"
        );
    }
}
