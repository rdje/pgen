# Held-carrier execution contract

Task: `PGEN-RGX-0078-0197`
Scope: read-only composition through `PGEN-RGX-0078-0196`; this file does not
authorize a product change.

## Accounting correction

The historical phrase **"102.138976561 ns accumulated strict" is
superseded**. The reproducible composition in `held_carrier_batch.txt` proves
that this number is a pre-implementation current-side target:

- **44.738976561 ns** is exact current-side work. Of that amount,
  **0.227347146 ns** is public telemetry whose removal is contract-blocked, so
  the exact-current eligible subtotal is **44.511629415 ns**.
- **57.400000000 ns** is caller-attributed region time, not an exact removable
  or net saving: G1-C atom-carrier work is 26.7 ns and memo segment-copy work
  is 30.7 ns.
- Every eligible candidate still owes its replacement instructions, codegen
  effects, ABI/register-pressure effects, and any interaction debit.
- Strict net saving banked before implementation and A/B measurement is
  therefore **0 ns**.

Gross and replacement-proxy views remain useful for prioritization but cannot
be promoted to measured net credit. The extra nonblocked gross/proxy views
through `-0196` bring the exploration target to **120.333328753 ns**; they do
not relax the acceptance gate.

## Binding acceptance ratchet

The director's 2026-07-20 rule applies to every regex-performance
implementation leaf:

1. Build the candidate and its immediate-parent baseline with the same
   release profile, fat LTO, allocator, generated regex parser, corpus, and
   probe configuration.
2. Run the canonical **2,189-cell PCRE2 external corpus** with the existing
   adaptive sampling policy. Preserve per-cell verdict and timing rows.
3. The candidate is accepted only when its unrounded external-corpus geomean
   is numerically and strictly lower than the immediate baseline:

   `candidate_external_corpus_geomean_ns < immediate_baseline_geomean_ns`

4. Equal or higher is an unconditional rejection. Revert the product source
   and generated-artifact candidate before committing the leaf; retain only
   the task/evidence/docs rejection record. A faster microbench, attributed
   model, or local cell cannot override an equal or slower corpus geomean.
5. Noise affects confidence reporting, never direction. Do not average away,
   round away, or reclassify an unfavorable result. A custody or thermal
   failure invalidates and reruns the sweep; it does not turn the result into
   an acceptance.
6. Speed is not sufficient: correctness/verdict identity is mandatory and the
   settled corpus MAX must remain **<= 483583 ns**. Any correctness change or
   MAX regression rejects the candidate even when the geomean falls.

The accepted immediate floor remains **1263.4 ns** until an implementation
leaf passes this ratchet and commits a lower measured value.

## One-fix, one-fresh-session protocol

Each implementation leaf owns exactly one optimization. After that fix has
been fully implemented, verified, corpus-measured, and either accepted or
reverted:

1. update its task leaf, evidence, live continuity docs, and mdBook when the
   behavior or developer surface requires it;
2. commit the accepted product change, or commit the rejection evidence with
   the product tree restored;
3. clear and verify `git_message_brief.txt`, prove the worktree clean, and
   stop;
4. start the next implementation fix only in a brand-new session.

This director rule overrides PNT/batch continuation between implementation
fixes. Read-only design slices may inventory later candidates, but may not
combine their code or measurements into one acceptance decision.

## Measurement custody

Each implementation leaf must pre-register and preserve:

- immediate baseline and candidate commit/source identities;
- generated regex artifact identity and the fat-LTO + mimalloc release-probe
  identities;
- canonical corpus path
  `regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl`;
- identical warmup, samples, slow-cell, and giant-cell settings;
- memory-guard, `caffeinate`, disk-floor, idle-host, and single-heavy-runner
  custody;
- base/candidate execution order, preferably order-swapped when the existing
  runner permits it without combining results across different candidates;
- full per-cell JSONL, unrounded geomean-of-`min_ns`, settled MAX, and verdict
  comparison.

Only a failed custody condition permits a rerun. An unfavorable but valid
measurement is the leaf's adjudication.

## Dependency and overlap map

- **G3 reset is independent and first.** The generated
  `prepare_parse_state` currently snapshots facts, replaces the whole
  `SemanticRuntimeState`, re-pushes facts, and clones predicate definitions.
  Because `semantic_runtime_state_mut()` is public, skipping reset on a first
  parse is unsound: callers may have mutated any state category. The compatible
  mechanism is an in-place reset that preserves only facts/their indices and
  predicate definitions while restoring every other field to `new()` state.
- **Checkpoint compaction is independent.** `scope_len` duplicates
  `chain_len`; the public `scope_len()` accessor can return `chain_len`.
- **Recursion-name work is one representation fix.** Fixed entry/exit,
  rollback, and growth are three costs of the same generated ID-only name
  carrier and must be measured together, not as three code fixes.
- **Bare diagnostics require an observer design.** The trace-off fast path may
  not silently remove public diagnostics or trace text.
- **G1-B needs an internal/public error boundary.** Public `ParseError` owns
  rich contextual data; the plausible fix is a drop-free internal cascade
  control error converted only at the public boundary, not an ABI-breaking
  public index.
- **Carrier core is coupled.** G1-C, memo segment-copy elision, packed events,
  and the unified derivation tape share representation/range/copy decisions.
  They require one design before implementation so the same copy or metadata
  lane is never credited twice.
- **Public telemetry stays blocked.** Its 0.227347146 ns is excluded unless a
  later design preserves post-parse observer compatibility.
- Thin lookup, immutable input view, and position forwarding remain gross or
  proxy views and must be repriced against whatever representation has landed
  before they can become implementation leaves.

## Ordered implementation leaves

1. `PGEN-RGX-0078-0198` — in-place semantic-runtime reset preserving facts,
   fact indices, and predicate definitions; exact-current target 22.563 ns.
2. `PGEN-RGX-0078-0199` — remove the duplicate checkpoint word while preserving
   the public accessor; exact-current target 0.859310862 ns.
3. `PGEN-RGX-0078-0200` — generated ID-only recursion-name path covering fixed,
   rollback, and growth work together; exact-current target 8.094507402 ns.
4. `PGEN-RGX-0078-0201` — bare diagnostic observability design, then one fix;
   exact-current target 8.042632 ns before replacement cost.
5. `PGEN-RGX-0078-0202` — drop-free internal cascade control error design,
   then one fix; exact-current target 4.422 ns before replacement cost.
6. `PGEN-RGX-0078-0203` — carrier-core design and implementation only after
   its joint replacement/overlap contract is complete.

Every numbered implementation stops at its own clean committed boundary and
requires the next fresh session before the following fix begins.
