// `.87`(d) C3: LOCATE the spawn refusal boundary instead of sampling a grid, and
// re-time the small sizes with repeats to test the warm-up explanation.
fn spawns(bytes: usize) -> bool {
    match std::thread::Builder::new().stack_size(bytes).spawn(|| {}) {
        Ok(h) => { let _ = h.join(); true }
        Err(_) => false,
    }
}
fn main() {
    const GIB: usize = 1024 * 1024 * 1024;
    // Bisect the largest spawnable reservation between a known-OK and a known-FAIL point.
    let (mut lo, mut hi) = (64 * 1024 * GIB, 128 * 1024 * GIB); // 64 TiB OK, 128 TiB FAIL
    assert!(spawns(lo), "premise: 64 TiB spawns");
    assert!(!spawns(hi), "premise: 128 TiB does not");
    while hi - lo > GIB {
        let mid = lo + (hi - lo) / 2;
        if spawns(mid) { lo = mid } else { hi = mid }
    }
    println!("largest spawnable stack : {} B = {:.3} TiB", lo, lo as f64 / (1024.0 * GIB as f64));
    println!("smallest refused stack  : {} B = {:.3} TiB", hi, hi as f64 / (1024.0 * GIB as f64));
    println!("host VA hypothesis 128 TiB = {} B", 128usize * 1024 * GIB);
    println!("=> a host of {:.2} PiB is the smallest that would derive a REFUSED reservation",
             (hi * 64) as f64 / (1024.0 * 1024.0 * GIB as f64));
    println!();
    // Repeats at the sizes that matter, to separate warm-up from a size effect.
    for bytes in [96 * 1024 * 1024usize, 256 * 1024 * 1024, 384 * 1024 * 1024, 16 * 1024 * GIB] {
        let mut v: Vec<u128> = (0..9).map(|_| {
            let t = std::time::Instant::now();
            assert!(spawns(bytes));
            t.elapsed().as_nanos()
        }).collect();
        v.sort_unstable();
        println!("{:>14} B  n=9  min {:>9} ns  median {:>9} ns  max {:>9} ns",
                 bytes, v[0], v[4], v[8]);
    }
}
// Falsify the EXPLANATION, not just the control (-0158's rule).
// Story: 16 TiB failed only because the process had already mapped near 64 TiB.
// If true, 16 TiB succeeds repeatedly in a FRESH process and fails only AFTER a big map.
fn spawns(bytes: usize) -> bool {
    match std::thread::Builder::new().stack_size(bytes).spawn(|| {}) {
        Ok(h) => { let _ = h.join(); true }
        Err(_) => false,
    }
}
fn main() {
    const GIB: usize = 1024 * 1024 * 1024;
    let arg = std::env::args().nth(1).unwrap_or_default();
    if arg == "fresh" {
        // ARM A — no prior large mapping.
        print!("fresh process, 16 TiB x8: ");
        for _ in 0..8 { print!("{} ", if spawns(16 * 1024 * GIB) { "OK" } else { "FAIL" }); }
        println!();
        print!("fresh process, 384 MiB x8 after those: ");
        for _ in 0..8 { print!("{} ", if spawns(384 * 1024 * 1024) { "OK" } else { "FAIL" }); }
        println!();
    } else {
        // ARM B — one 64 TiB spawn/join first, then the same 16 TiB requests.
        println!("64 TiB first: {}", if spawns(64 * 1024 * GIB) { "OK" } else { "FAIL" });
        print!("then 16 TiB x8: ");
        for _ in 0..8 { print!("{} ", if spawns(16 * 1024 * GIB) { "OK" } else { "FAIL" }); }
        println!();
        print!("then 384 MiB x8: ");
        for _ in 0..8 { print!("{} ", if spawns(384 * 1024 * 1024) { "OK" } else { "FAIL" }); }
        println!();
    }
}
// Bounded: the realistic embedder question only. How many CONCURRENT dedicated parse
// threads of a given reservation can be live at once?
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
fn max_concurrent(bytes: usize, cap: usize) -> usize {
    let go = Arc::new(AtomicBool::new(false));
    let mut live = Vec::new();
    while live.len() < cap {
        let g = Arc::clone(&go);
        match std::thread::Builder::new().stack_size(bytes).spawn(move || {
            while !g.load(Ordering::Relaxed) { std::hint::spin_loop(); }
        }) {
            Ok(h) => live.push(h),
            Err(_) => break,
        }
    }
    let n = live.len();
    go.store(true, Ordering::Relaxed);
    for h in live { let _ = h.join(); }
    n
}
fn main() {
    const GIB: usize = 1024 * 1024 * 1024;
    for (label, bytes, cap) in [
        ("256 MiB (reference)", 256 * 1024 * 1024usize, 128usize),
        ("384 MiB (this host, .87(d))", 384 * 1024 * 1024, 128),
        ("16 GiB (a 1 TiB host)", 16 * GIB, 128),
    ] {
        let n = max_concurrent(bytes, cap);
        println!("{:>30}  concurrent live: {:>4}{}  = {:.2} TiB VA",
                 label, n, if n == cap { " (probe cap)" } else { " ** REFUSED **" },
                 (n * bytes) as f64 / (1024.0 * GIB as f64));
    }
}
