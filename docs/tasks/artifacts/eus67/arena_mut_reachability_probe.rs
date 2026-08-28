//! ENGINE-UNIVERSAL-SERVICES.67(b) — CAN THE ARENA BE WALKED AFTER THE PARSE? THREE SHAPES, THREE
//! REFUSALS. This probe is the evidence for that claim; `-0122` had priced the walk as merely
//! needing the parser dropped first, and this is what refutes it.
//!
//! It mimics `NodeArena`'s UNIFIED-lifetime shape exactly — `Arena<Node<'a>>` handed out as
//! `&'a Arena<'a>` — which is the property that makes the arena permanently shared-borrowed:
//! a local of type `Arena<'a>` needs `'a` live for its whole scope, so the `&'a` borrow the
//! consumer holds can never end while the arena exists.
//!
//! RUN IT (from the repository root; the rlib path is whatever the release build produced):
//!
//! ```text
//! rustc --edition 2021 -O \
//!   --extern typed_arena=rust/target/release/deps/libtyped_arena-<hash>.rlib \
//!   -L rust/target/release/deps \
//!   -o tmp/eus67b/probe docs/tasks/artifacts/eus67/arena_mut_reachability_probe.rs
//! ```
//!
//! MEASURED 2026-08-29, rustc stable-aarch64-apple-darwin:
//!   variant_b (parser confined to an inner scope, then `&mut`) -> E0502
//!   variant_c (consume the arena by value, `into_vec`)         -> E0505
//!   variant_d (`&self` accumulation at alloc time)             -> COMPILES, and is the shape shipped
//! A fourth, `drop(parser)` then `&mut`, also answers E0502 — same reason as variant_b.
//!
//! ⛔ EXPECTED TO FAIL TO COMPILE AS WRITTEN. Comment out `variant_b`/`variant_c` to watch
//! `variant_d` build and run; that asymmetry IS the result.
use std::cell::Cell;

struct Node<'a> {
    kids: Vec<&'a Node<'a>>,
}

struct Arena<'a> {
    nodes: typed_arena::Arena<Node<'a>>,
    owned: Cell<usize>,
}

impl<'a> Arena<'a> {
    fn new() -> Self {
        Arena { nodes: typed_arena::Arena::new(), owned: Cell::new(0) }
    }

    /// Variant D, and the only one that compiles: accumulate on the `&self` path.
    fn alloc(&'a self, n: Node<'a>) -> &'a mut Node<'a> {
        self.owned.set(self.owned.get() + n.kids.capacity() * 8);
        self.nodes.alloc(n)
    }

    /// Variant B's requirement: `typed_arena::Arena::iter_mut` needs `&mut self`.
    fn deep_mut(&mut self) -> usize {
        self.nodes.iter_mut().map(|n| n.kids.capacity() * 8).sum()
    }

    /// Variant C's requirement: consuming the arena needs it un-borrowed.
    fn deep_owned(self) -> usize {
        self.nodes.into_vec().iter().map(|n| n.kids.capacity() * 8).sum()
    }

    fn deep_cell(&self) -> usize {
        self.owned.get()
    }
}

struct Parser<'a> {
    arena: &'a Arena<'a>,
}

impl<'a> Parser<'a> {
    fn new(arena: &'a Arena<'a>) -> Self {
        Parser { arena }
    }
    fn run(&mut self) {
        let _ = self.arena.alloc(Node { kids: vec![] });
    }
}

/// B — the parser is confined to an inner scope and is provably dead here. Still `E0502`.
fn variant_b() -> usize {
    let mut arena = Arena::new();
    {
        let mut p = Parser::new(&arena);
        p.run();
    }
    arena.deep_mut()
}

/// C — consume the arena instead of borrowing it mutably. `E0505`.
fn variant_c() -> usize {
    let arena = Arena::new();
    {
        let mut p = Parser::new(&arena);
        p.run();
    }
    arena.deep_owned()
}

/// D — accumulate as we allocate. Compiles, and is what `NodeArena::record_owned_heap` does.
fn variant_d() -> usize {
    let arena = Arena::new();
    {
        let mut p = Parser::new(&arena);
        p.run();
    }
    arena.deep_cell()
}

fn main() {
    println!("{} {} {}", variant_b(), variant_c(), variant_d());
}
