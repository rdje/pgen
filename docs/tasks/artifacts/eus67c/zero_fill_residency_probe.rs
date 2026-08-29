//! ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — the FIX PRICING for the dense thin-memo table.
//!
//! ⛔ THE QUESTION, AND IT IS A RESIDENCY QUESTION, NOT AN ALLOCATION ONE.
//! `.67`(c)(ii) measured the table at 98.4 % of the marginal RESIDENT cost: 914 362 632
//! bytes allocated, 112 bytes written, 906 MB resident. The cause named there was
//! `Vec::resize(need, 0)`, which faults in every page because it WRITES every slot.
//! `.67`(c)(i) measured occupancy at 0.07 %-4.6 %, so 95-99.9 % of those pages carry a
//! zero that the generation stamp already makes unnecessary — an unwritten slot is
//! invalid whatever it contains.
//!
//! The candidate fix is therefore to obtain the table from a ZEROED allocation the
//! kernel backs lazily, so untouched pages never become resident. In Rust that is
//! `vec![0u64; n]`, which specialises to `alloc_zeroed` for an integer element, against
//! `Vec::resize(n, 0)`, which allocates uninitialised and then writes.
//!
//! ⛔ WHETHER THAT WORKS IS A PROPERTY OF THE HOST'S ALLOCATOR, NOT OF THE SOURCE, so
//! it is measured rather than reasoned about. This probe isolates the mechanism from
//! the engine entirely: no parser, no arena, no memo — one table, one fill strategy,
//! one occupancy, and the peak RSS the host reports for it.
//!
//! ⛔ AND IT IS DELIBERATELY NOT AN ENGINE CHANGE. It prices the mechanism. Whether the
//! engine can adopt it is a separate question, because `ThinMemoScratchLease` RECYCLES
//! the table across parses and a recycled table is already faulted in.
//!
//! Build:  rustc -O -o tmp/eus67c/zero_fill_residency_probe \
//!             docs/tasks/artifacts/eus67c/zero_fill_residency_probe.rs
//! Usage:  zero_fill_residency_probe <resize|zeroed|reserve_only> <slots> <written> [stride_slots]
//!
//! ⛔⛔ **THE STRIDE IS A PARAMETER BECAUSE THE ANSWER IS A BRACKET, NOT A NUMBER.** What a
//! lazily-backed table actually saves depends on how many distinct PAGES the writes touch,
//! and that is a property of the write pattern — `slot = rule_index * (input_len + 1) +
//! position` — which nothing in this repository measures. So the two ends are run
//! explicitly and the real answer is stated as lying between them:
//!   `stride = 1`   CLUSTERED   — the writes are contiguous; pages touched ~= occupancy
//!   `stride = 512` SCATTERED   — one write per 4 KiB page; every page is touched, and a
//!                                lazy table saves NOTHING. This end must be run, because
//!                                an experiment that cannot come out against the fix is
//!                                not a measurement of it.
//! The third point is `written = 14`, the value `.67`(c)(ii) MEASURED on its isolating
//! comment-only shape, so one row of this table is directly comparable to the 906 MB
//! already published.

use std::env;

/// 4 KiB pages / 8 bytes per slot — the stride that maximises pages touched per write.
const SLOTS_PER_PAGE: usize = 512;

fn main() {
    let args: Vec<String> = env::args().collect();
    if !(4..=5).contains(&args.len()) {
        eprintln!(
            "usage: {} <resize|zeroed|reserve_only> <slots> <written> [stride_slots]",
            args[0]
        );
        std::process::exit(2);
    }
    let arm = args[1].clone();
    let slots: usize = args[2].parse().expect("slots must be an integer");
    let written: usize = args[3].parse().expect("written must be an integer");
    let stride: usize = args
        .get(4)
        .map(|s| s.parse().expect("stride must be an integer"))
        .unwrap_or(SLOTS_PER_PAGE)
        .max(1);

    let mut table: Vec<u64> = match arm.as_str() {
        // The engine's shape today: `ThinMemoScratch::begin` does `slots.resize(need, 0)`.
        "resize" => {
            let mut v: Vec<u64> = Vec::new();
            v.resize(slots, 0);
            v
        }
        // The candidate: an integer `vec![0; n]` specialises to `alloc_zeroed`.
        "zeroed" => vec![0u64; slots],
        // ⛔ THE CONTROL ARM. Reserving without initialising isolates how much of the
        // arm's RSS is the ALLOCATION and how much is the ZERO-FILL. If `resize` and
        // `reserve_only` read the same peak RSS, the fill is not what makes the pages
        // resident and this whole fix direction is refuted — which is exactly what a
        // control is for.
        "reserve_only" => {
            let v: Vec<u64> = Vec::with_capacity(slots);
            v
        }
        other => {
            eprintln!("unknown arm: {other}");
            std::process::exit(2);
        }
    };

    // Write `written` slots one page apart, in the encoding the real table uses:
    // `(generation as u64) << 32 | (idx + 1)`, never zero.
    let cap = table.capacity();
    let mut touched = 0usize;
    let mut pages = std::collections::BTreeSet::new();
    for i in 0..written {
        let slot = i.wrapping_mul(stride) % cap.max(1);
        if arm == "reserve_only" {
            // The reserve arm has len 0; writing through the pointer is what the
            // engine would do only after a resize, so this arm writes nothing and
            // measures the allocation alone.
            break;
        }
        table[slot] = (1u64 << 32) | (i as u64 + 1);
        pages.insert(slot / SLOTS_PER_PAGE);
        touched += 1;
    }

    // Keep the table alive past the measurement and force the writes to be observable,
    // so nothing here can be optimised away.
    // ⛔ THE SAME STRIDE THE WRITES USED, or the checksum reads slots nothing wrote and
    // comes out 0 — which looks identical to the writes having been optimised away, the
    // one failure this line exists to make impossible.
    let checksum: u64 = (0..touched.min(1024))
        .map(|i| table[i.wrapping_mul(stride) % cap.max(1)])
        .fold(0u64, |a, b| a ^ b);

    // ⛔ `distinct_pages` is COUNTED, not derived from `written / stride`: the modulo wraps
    // and revisits pages, which is precisely the effect that makes a scattered write pattern
    // touch every page at an occupancy of a few per cent.
    println!(
        "ZERO-FILL-PROBE: arm={} slots={} table_bytes={} written={} stride={} touched={} \
         distinct_pages={} table_pages={} page_coverage={:.6} checksum={}",
        arm,
        slots,
        slots * std::mem::size_of::<u64>(),
        written,
        stride,
        touched,
        pages.len(),
        slots.div_ceil(SLOTS_PER_PAGE),
        pages.len() as f64 / slots.div_ceil(SLOTS_PER_PAGE).max(1) as f64,
        checksum,
    );
    // `table` is dropped after the print, so peak RSS covers the whole live window.
    drop(table);
}
