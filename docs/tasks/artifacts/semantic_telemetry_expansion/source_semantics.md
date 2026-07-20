# Semantic-store counter reader and observability audit

## Semantic classification

`SemanticStoreCounters` is documented as cumulative diagnostic observation of
store operations (`semantic_runtime.rs:2250-2263`). Its fields fall into two
classes:

- Diagnostic only: `facts_emitted`, `facts_imported`, `facts_rolled_back`,
  `scopes_opened`, `scopes_closed`, `rollbacks`, `rollbacks_unchanged`,
  `rollbacks_tournament`, `rollbacks_tournament_unchanged`, and
  `rollbacks_nonempty_chain`.
- Required semantic state: `predicate_evaluations`. A memoized body that
  consulted mutable store state cannot receive a store-blind cache stamp;
  generated and interpreter memo paths snapshot/compare this monotone value.

The only non-test readers of the diagnostic fields in the repository are the
rule-outcome JSON dump paths in `parser_registry.rs:278-296,330-381,775-805`.
Those paths opt into coverage before parsing, which already makes
`bare_parse=false` and routes through the observability protocol twin. No
diagnostic counter steers a parse verdict, store mutation, rollback, memo
validity, or replay.

## Exact bare-target mechanism

In the preserved optimized binary, the hot unchanged-rollback path folds the
four rollback-classification counters into two vectorized forms at the common
parser-relative base materialization `add ..., x20, #0x228`:

- 39 ordinary rollback sites increment `rollbacks` + `rollbacks_unchanged` in
  one 128-bit load/add/store sequence (6 instructions); and
- 24 tournament-cleanup sites increment those two plus
  `rollbacks_tournament` + `rollbacks_tournament_unchanged` in one 256-bit
  load/add/store sequence (7 instructions).

The required write-epoch/deferred-obligation fast-path checks precede each
sequence and are excluded. The conditional `rollbacks_nonempty_chain` update
follows and was already fully owned by `PGEN-RGX-0078-0185`; it is excluded
here. Slow mutation rollback, fact/scope operations, and required memo-taint
updates are outside the two sampled fused symbols and receive no new price.

## Public observability boundary

The rollback quartet is outcome-neutral but is not silently removable under
the current API contract. Generated parsers expose
`semantic_runtime_state()`, and `SemanticRuntimeState::counters()` publicly
returns the cumulative values. Unlike generated rule-call counters, this
accessor has no observer latch that can influence the next parse's twin
selection. An external embedder can therefore parse first and inspect
counters afterward.

The maintained outcome-dump path is safe for a future bare specialization
because it observes/baselines before the parse and already forces the protocol
twin. External after-the-fact observation is not. Any future implementation
must do one of the following explicitly and document/test it:

1. preserve all semantic counters on the bare path;
2. introduce a pre-parse opt-in and make counters explicitly opt-in rather than
   silently cumulative; or
3. define a separate non-observable internal state for the optimized path and
   retain the public contract through the protocol route.

This leaf licenses none of those surface changes. It prices the exact ordinary
bare work only to decide whether a larger design investment is justified.
