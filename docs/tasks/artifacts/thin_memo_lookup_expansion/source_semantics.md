# Thin-memo lookup source contract and candidate representation

## Required behavior

The fused cyclic-spine memo is required for recursion safety and replay, not a
diagnostic cache. The current emitter:

- constructs a constant-rule/variable-position key and probes the memo
  (`cascade.rs:619-623`);
- validates pure/store-read stamps against the semantic store
  (`cascade.rs:624-650`);
- replays a success segment or cached failure (`cascade.rs:633-647`);
- removes a stale entry (`cascade.rs:651-653`); and
- inserts success/failure entries only for non-mutating bodies
  (`cascade.rs:669-718`).

None of stamp validation, segment replay, stale invalidation, or insertion is
removable. The shipped generator emits one `get` and one `remove` site for each
of 28 cyclic fused internal rules. The separately accepted thin-memo-success
ranges own the success segment-copy/insert work and must not be counted again.

## Concrete candidate

Replace the shared `FxHashMap<(RuleId, usize), ThinDerivSegMemoEntry>` with:

1. one generated `u32` slot row per cycle-participating fused rule, indexed by
   input position (`0` = absent, `n + 1` = dense-entry index); and
2. a dense `Vec<Option<ThinDerivSegMemoEntry>>` or equivalent stable entry arena.

Each generated match site is monomorphic in `RuleId`, so it selects its row at
compile time and indexes only by `position`. A hit loads the entry index and
then executes today's unchanged stamp/replay logic. Stale invalidation clears
the slot and retires or tombstones the dense entry. Insert appends/reuses an
arena entry and publishes its index in the slot.

This removes Fx hash constant construction/mixing, control-byte SIMD probing,
probe loops, and `(RuleId, position)` key comparison. It does **not** remove a
presence branch, slot load, dense-entry base/address calculation, row
initialization, or stale-entry lifetime work. A direct array of full
`Option<ThinDerivSegMemoEntry>` values is rejected: multiplying the large
inline-small payload by every rule/position cell would make initialization and
memory footprint structurally unsafe. The compact `u32` indirection is the
only representation considered by this pricing leaf.

## Pricing boundary

The preserved binary has five complete lookup sites in the two sampled target
symbols. Each exact 52-instruction range starts at `[x20,#0x490]` occupancy and
ends after the second hashbrown control-byte probe loop; hit stamp processing,
miss body entry, stale removal, and all accepted success insertion ranges are
outside.

The replacement proxy retains only the current three parser-relative table
state loads and initial occupancy branch per site. This is deliberately
optimistic: it prices the candidate's mandatory slot load, dense-entry
base/address work, and initialization at zero. Therefore the resulting
hash-exclusive 3.824606730 ns is an upper bound on honest capture, not a
prediction.

At 30% capture the held bundle plus that optimistic upper bound reaches
28.875671619 ns, only 0.075671619 ns above the 28.8 ns noise floor. That sliver
is 0.26% of the noise floor and disappears as soon as any omitted replacement
work is priced. It is not honest implementation margin.
