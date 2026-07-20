# Source-semantics audit for the preserved machine roles

This audit corrects a semantic label; it does not transfer current Rust field
offsets into the older preserved probe. The preserved binary independently
proves the `0x248` increment and `0x250` snapshot/compare machine shapes.

## `predicate_evaluations` is required memo state

- `rust/src/ast_pipeline/semantic_runtime.rs:2302-2314` defines
  `predicate_evaluations` as the memo **taint signal**. It is cumulative across
  rollback so a predicate evaluated inside nested speculation still taints the
  enclosing attempt.
- `rust/src/ast_pipeline/semantic_runtime.rs:2631-2640` states the operational
  contract: `memoized_call` snapshots the count around a rule body; a changed
  count means the body consulted mutable store and neither success nor failure
  may be cached under a store-blind `(rule, position)` key.
- `rust/src/ast_pipeline/ast_based_generator/cascade.rs:654-679` emits exactly
  that snapshot/compare and withholds the thin-memo stamp when the count
  changes.

Therefore preserved offset `0x250` is not diagnostic-only and is not removable
without a replacement memo-soundness design. Its `9 / 13 / 16` samples are
retained as required work.

## `rollbacks_nonempty_chain` is diagnostic-only

- `rust/src/ast_pipeline/semantic_runtime.rs:2296-2301` calls
  `rollbacks_nonempty_chain` a per-cell scope-exposure diagnostic.
- `rust/src/ast_pipeline/semantic_runtime.rs:3208-3220` and `:3267-3276`
  increment it after testing `checkpoint.chain_len > 1`; the value does not
  steer rollback or parse semantics.
- Its readers in `rust/src/parser_registry.rs:288-296`, `:336-344`, and
  `:373-381` only serialize diagnostic outcome dumps.

Therefore preserved offset `0x248` and its exact chain-depth gate are removable
from a compile-time bare specialization. The other rollback counters were not
part of `PGEN-RGX-0078-0184`'s sampled-field claim and remain outside this
slice; they are durably queued as `PGEN-RGX-0078-0187`, not silently absorbed
here.

## Coverage and trace ownership

- Generated `try_parse` snapshots the transactional coverage length and
  conditionally truncates it on rollback. Coverage is disabled for ordinary
  parsing, so the preserved hot path still executes a redundant zero-length
  snapshot/compare/store protocol.
- The cached `logger_enabled` byte gates cold trace arms. A compile-time bare
  path can remove the gate itself while retaining the existing instrumented
  path byte-for-byte.

`classify_bare_diagnostics.py` establishes the complete machine ownership and
exclusions. In particular, eight coverage checkpoint values share an `stp`
with a required word; those whole instructions are not priced because the
required neighbor still needs a store.
