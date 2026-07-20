# Semantic-checkpoint field audit

## Custody and scope

`inspect_checkpoint_word.py` refuses unless all of these remain exact:

- `rust/src/ast_pipeline/semantic_runtime.rs` SHA-256
  `26ddf12ea89927a07f5d0f8acb79e8e5162cf0f0a32099dcb3746e35552f5701`;
- preserved probe SHA-256
  `1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8`;
- all three raw-PC capture hashes inherited from `PGEN-RGX-0078-0182`;
- the exact seven-field declaration, lockstep mutation census, compiler
  version, layout-probe hash, target symbol starts, and prior-mechanism ranges.

This is a read-only feasibility result. It changes no runtime, emitter, or
generated parser.

## Every field and reader

The checkpoint is currently seven 64-bit words:

| Field | Required reader class | Adjudication |
| --- | --- | --- |
| `scope_len: usize` | `scope_len()`; release `commit`; debug-only rollback assertions | Redundant with `chain_len` by the invariant below. |
| `fact_len: usize` | export accessor; delta slices; rollback truncate/index repair; commit | Required and arbitrary-width. |
| `deferred_len: usize` | empty-delta/no-op guards; delta slice; rollback truncate | Required and arbitrary-width. |
| `scope_arena_len: usize` | accessor; delta slice; rollback truncate; commit | Required and arbitrary-width. |
| `chain_len: usize` | rollback diagnostic, fast/slow debug assertions | Required: it preserves the checkpoint-time depth even after later mutations. |
| `chain_trail_len: usize` | unchanged proof, checkpoint-chain reconstruction, trail unwind | Required and arbitrary-width. |
| `write_epoch: u64` | empty-delta and no-op rollback guards | Required monotone epoch; narrowing introduces wrap/collision risk. |

No observational or corpus-derived cap is admitted. Facts, deferred work,
arena entries, active depth, and trail length are all `usize` populations, and
the epoch is deliberately monotone `u64`. Packing or narrowing any of those
six retained words would add a semantic bound that does not exist today.

## Exact equivalence proof

`scopes.len() == active_chain.len()` is an inductive state invariant:

1. `SemanticRuntimeState::new` creates exactly one global scope frame and
   exactly one `ScopeId::ROOT` chain entry.
2. `open_scope` pushes one chain id and one scope frame.
3. A successful `close_scope` pops one chain id and one scope frame; a failed
   close mutates neither.
4. `extract_delta_since_slow` captures `final_active_chain` and `final_scopes`
   from the same invariant state. `apply_delta` installs both captured values.
5. Slow rollback first restores `active_chain`, then rebuilds `scopes` by
   mapping exactly one frame from every restored chain id.
6. Both fields are private. The source-pinned mutation census finds no other
   push, pop, or assignment path.

Therefore every checkpoint is born with
`scope_len == chain_len`. The legacy accessor can return `chain_len`; the
release `commit` comparison can use `chain_len` against either current mirror;
and debug assertions can compare the current mirrors against `chain_len`.
Removing `scope_len` loses no state, changes no counter meaning, and imposes no
cap.

## Machine census and the corrected 40th site

The prior `PGEN-RGX-0078-0183` detector intentionally anchored each checkpoint
to a nearby `"Starting speculative parse"` trace literal and therefore found
39 ordinary inlined `try_parse` checkpoints. A complete scan for the first
physical checkpoint field (`ldr ..., [x20, #0xd8]`) finds **40**, not 39.

The missing site is in `cascade_match_atom{closure}`:

- source loads: `0x100073f40..0x100073f64`;
- seven contiguous stack stores: `0x100073f68..0x100073f80`,
  `sp+0x560..sp+0x590`;
- no entry-trace literal or logger gate in that materialization;
- the same tournament region later calls `extract_delta_since` at
  `0x1000751a4` and `rollback_to_labeled` at `0x10007526c`.

This is the trace-less outer C3-B tournament checkpoint emitted by the
`tournament_semantic_checkpoint` path, not another ordinary `try_parse` entry.
The old trace-anchored result was a complete census of its stated anchor, but
not a complete census of checkpoints. The correction changes the old
stack-carrier attribution from **68/66/74 checkpoint + 699/624/727 unmatched**
to **68/67/75 checkpoint + 699/623/726 unmatched**; every band still re-sums
to the same 902/848/987 stack residual. The newly reclassified store is the
required `chain_trail_len` word at `0x100073f7c`, sampled 0/1/1. It is not part
of the removable `scope_len` word and changes no past price.

Across all 40 sites there are 40 `scope_len` source loads and 40 word-zero
stores. Twenty-eight stores are exclusive one-word `str` instructions;
12 share an `stp` with required `fact_len`. The complete seven-word initial
materializations use 246 instructions and sample 68/67/75. Later aggregate
copies, reloads, argument packing, and callee-side layout effects are coupled
to the candidate's six-word ABI and receive zero strict credit without a
candidate binary.

## Lossless replacement and exact price

The concrete common carrier is the existing field sequence with only
`scope_len` removed. Under the pinned compiler it is 48 bytes, align 8,
`Copy`, and drop-free, versus 56 bytes today.

Strictly removable current instructions are:

- all 40 separate parser-field source loads: samples **5/7/7**,
  **0.316397910 ns**;
- the 28 exclusive word-zero stores: samples **11/13/6**,
  **0.542912952 ns**.

Strict total: **0.859310862 ns**. The 12 paired stores remain instructions
because `fact_len` must still be written. Crediting their full 1/0/0 sample as
if repacking removed the instruction is an explicitly optimistic
**0.021317526 ns** ceiling, for **0.880628388 ns** total.

Adding the strict value to the pre-slice **101.279665699 ns** bundle gives
**102.138976561 ns**. At 30% capture that is **30.641692968 ns**, only
**1.841692968 ns** above the 28.8 ns noise floor. The optimistic view leaves
only **1.848088226 ns**. Both exclude mandatory candidate-side layout/ABI/
register-pressure effects, so implementation remains **HOLD**.
