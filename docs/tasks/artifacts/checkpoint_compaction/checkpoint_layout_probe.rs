use std::mem::{align_of, needs_drop, size_of};

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Checkpoint7 {
    scope_len: usize,
    fact_len: usize,
    deferred_len: usize,
    scope_arena_len: usize,
    chain_len: usize,
    chain_trail_len: usize,
    write_epoch: u64,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Checkpoint6 {
    fact_len: usize,
    deferred_len: usize,
    scope_arena_len: usize,
    chain_len: usize,
    chain_trail_len: usize,
    write_epoch: u64,
}

fn main() {
    println!(
        "checkpoint7 size={} align={} drop={}",
        size_of::<Checkpoint7>(),
        align_of::<Checkpoint7>(),
        needs_drop::<Checkpoint7>(),
    );
    println!(
        "checkpoint6 size={} align={} drop={}",
        size_of::<Checkpoint6>(),
        align_of::<Checkpoint6>(),
        needs_drop::<Checkpoint6>(),
    );
}
