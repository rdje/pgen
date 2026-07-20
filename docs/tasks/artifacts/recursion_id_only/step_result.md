# PGEN-RGX-0078-0200 — step result

Session #175, 2026-07-20. One implementation fix per the fresh-session
directive: the generated ID-only recursion-name path (the `-0197` unit fusing
`name_fixed` 3.940409443 + `name_rollback` 0.149032382 + `name_growth`
4.005065577 = **8.094507402 ns** exact-current pricing).

## Verdict: LAND (the third fix through the `-0197` strict-geomean ratchet)

| gate | result |
|---|---|
| corpus geomean (unrounded, same-session) | **1198.8631388258577 → 1153.517986985003 ns = −3.7823%** — strict decrease PASS |
| verdict flips | **0 / 2,189** |
| corpus MAX | 451,833 ≤ settled 483,583 ns (same worst cell `line_725`) — PASS |
| floor validation | base bench geomean 1921.2 vs banked 1937.4 ns (−0.84%) — custody OK |
| bench (steering only) | 1927.6 → 1818.3 ns (−5.67%); every pattern faster; `digit_sequence` −13.14% |

Accepted floor re-baselined: corpus geomean **1,193.2 → 1,153.5 ns**
(vs the `-0199` banked floor; the same-session base sweep read 1,198.9 ns,
+0.48% session drift, inside noise — the strict compare is same-session by
design). Bench floor re-baselined ≈1,937.4 → ≈1,818.3 ns. Settled MAX bound
unchanged (483,583 ns; candidate worst cell 451,833 ns). The <1 µs call-off
bar now needs **−13.3%**.

Honest magnitude note (the `-0197` noise clause): the measured −45.3 ns is
≈5.6× the 8.094507402 ns exact-current classifier pricing. Unlike
`-0198`/`-0199`, the −3.78% magnitude sits OUTSIDE the ≈2.3% observed noise
span, so direction and magnitude agree this time. Plausible surplus sources
(unpriced by design in the conservative classifiers): the elided name-stack
`RawVec::grow_one` beyond the sampled bands, zero name-frame cache traffic in
the fused loops, and cross-site codegen/register-pressure effects of the
narrower bare frames — the same class the `-0199` leaf recorded.

## The change (one representation fix)

- Lib (`rust/src/ast_pipeline/mod.rs`): additive
  `RecursionGuard::enter_id_bare`/`exit_bare`/`truncate_stacks`;
  `check_cycle_id`'s maximum-depth arm re-pointed to `rule_id_stack.len()`
  (byte-identical wherever the stacks are lockstep — every protocol/legacy
  parse; exact under mixed stacks). All public legacy methods/fields
  verbatim. One focused test pins the bare-path semantics.
- Emitter (`ast_based_generator.rs`): the universal protocol `try_parse`
  takes TWO independent stack snapshots restored per-stack via
  `truncate_stacks` (closes the mixed-mode hazard; degenerate-identical in
  all-paired parses); a cascade-gated `try_parse_bare` does the ID-only
  snapshot/truncate with the rollback label mapped through `RULE_NAMES`;
  `create_contextual_error` reconstructs its rule-name vector from
  `rule_id_stack` through `RULE_NAMES` (identical by bijection; COMPLETE
  during bare parses).
- Cascade emitter (`cascade.rs`): the internal match frame pushes/pops
  ID-only (`enter_id_bare`/`exit_bare`); both cascade speculation emission
  sites call `try_parse_bare`; emitter self-check tests pin the bare names.

## Custody

- Base probe `8aef8541…` = the preserved `-0199` floor probe (cmp-verified
  pre-change), embedding regex artifact `f85f2121…` (hooks form).
- Candidate probe `92fba055…`, embedding regex artifact `b535be2c…`
  (hooks form at `-0200` vintage).
- All-11 regen train green incl. the ebnf fixed-point check; the
  expected-delta review passes over all 11 artifacts (`overall=0`,
  `artifact_delta_review.txt`).
- **Session custody finding** (design_prereg §5b, memory-recorded): the
  regex canonical artifact is the `--enable-parser-hooks` emit —
  `focus_regex` alone emits the typed-less default; the train's step 5b
  re-emits hooks at the canonical path. Surfaced to the director as a
  Makefile-gap follow-up (unowned by this leaf).
- Serialized memory-guarded caffeinated runs, order-alternated bench
  rounds, full per-cell JSONLs banked both sides.

## Battery

See the leaf's acceptance checklist (recorded post-battery, same commit).
