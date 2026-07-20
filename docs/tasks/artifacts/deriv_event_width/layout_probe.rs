use std::mem::{align_of, needs_drop, size_of};

// Exact source shape of `crate::ast_pipeline::DerivEvent` at the custody pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DerivEvent {
    OrWinner(usize),
    QuantCount(usize),
    OptPresent(bool),
    TokStart(usize),
    TokEnd(usize),
}

// Feasibility candidate: ordinary events occupy one machine word.  Tag 6 is
// an in-band escape marker carrying the original tag in its upper bits; the
// following word carries the full, unmodified usize payload.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PackedDerivWord(usize);

fn main() {
    println!(
        "DerivEvent size={} align={} needs_drop={}",
        size_of::<DerivEvent>(),
        align_of::<DerivEvent>(),
        needs_drop::<DerivEvent>(),
    );
    println!(
        "PackedDerivWord size={} align={} needs_drop={}",
        size_of::<PackedDerivWord>(),
        align_of::<PackedDerivWord>(),
        needs_drop::<PackedDerivWord>(),
    );
    println!("usize_bits={}", usize::BITS);
    println!("ordinary_payload_max={}", usize::MAX >> 3);

    // Keep every variant live so this probe cannot silently drift into a
    // declaration-only approximation that misses one source variant.
    std::hint::black_box([
        DerivEvent::OrWinner(0),
        DerivEvent::QuantCount(0),
        DerivEvent::OptPresent(false),
        DerivEvent::TokStart(0),
        DerivEvent::TokEnd(0),
    ]);
}
