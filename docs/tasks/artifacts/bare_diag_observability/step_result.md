# `PGEN-RGX-0078-0201` — result: **LANDED** (the fourth fix through the `-0197` strict-geomean ratchet)

## What landed

The pre-registered design (`design_prereg.md`) + exactly one fix:

- **LIB** (`semantic_runtime.rs`): additive `rollback_to_labeled_bare` — the hot
  rollback minus ONLY the diagnostic-only `rollbacks_nonempty_chain`
  classification (fast path inline; the outlined slow path gained a private
  `maintain_chain_diag` flag, observed entries pass `true`). The telemetry
  quartet and the REQUIRED `predicate_evaluations` memo-taint signal are
  maintained identically on both variants. The observed-parse boundary is
  documented on `counters()` + the field itself. One focused test
  (`bare_rollback_skips_only_the_chain_diagnostic_on_both_paths`).
- **EMITTER**: `try_parse_bare` no longer takes/restores the transactional
  coverage-length snapshot (provably dead on the bare path: every coverage
  push is `coverage_enabled`-gated and the fused graph emits no pushes) and
  rolls back through the bare twin; the island C3-B cleanup emissions
  (`cascade.rs`) use the bare twin. No routing-predicate change (no latch —
  the design's §4 adjudication).
- **DOCS**: TOOLBOX 3.5 observed-parse boundary note; the book's observability
  twin section gained the boundary paragraph (`inside-parser-performance.md`).

Design finding recorded: the C1-priced 170-logger-gate member was ALREADY
absorbed by `-0200`'s `try_parse_bare` (zero trace gates in the generated
cascade region) — no implementable logger-gate subset remained.

## Adjudication (binding `-0197` ratchet — `adjudication.txt`)

- Corpus (2,189 cells): unrounded geomean **1176.9554385111642 →
  1163.911714168528 ns = −1.1083%** — strict same-session decrease ⇒ **LAND**.
- Verdict flips **0/2,189**; candidate MAX **462,750 ≤ 483,583 ns** settled
  bound (same worst cell `line_725`).
- Floorval custody: base bench geomean 1847.7 vs banked 1840.4 = **+0.40%** OK.
- Bench steering: geomean 1826.8 → 1805.3 ns (**−1.17%**); `literal_simple`
  −7.58%, `alternation` −3.05%, `capture_groups` −1.30%, `character_class`
  −1.02%, `url_simple` +3.93% (single-pattern jitter), rest flat.
- Honest magnitude note: −1.11% is INSIDE the ≈2.3% observed noise span — the
  ratchet's strict same-session decrease adjudicates (the `-0198`/`-0199`
  precedent); the pre-registered model predicted a sub-noise single-digit-ns
  win, so direction and magnitude are consistent with the pricing.

## Floor re-baseline (cross-session drift stated explicitly)

Floor of record moves to the candidate's same-session reading: corpus geomean
**1,163.9 ns** (unrounded 1163.911714168528), bench ≈**1,805.3 ns**, settled
MAX bound unchanged (483,583 ns). The nominal rise vs the banked `-0200` floor
(1,153.5) is CROSS-SESSION DRIFT, not regression: the no-perf-change probe
lineage read 1,153.5 (`92fba055`, session #175) → 1,172.1 (the PROVEN-NEUTRAL
`948cbd63` hook-removal rebuild, same day) → 1,176.96 (the byte-identical
`948cbd63` re-read, this session) = +2.0% cumulative with no accepted perf
change, while this fix measured **−1.1083% same-session**. The <1 µs closure bar now needs −14.1%
from the floor-of-record reading.

## Custody

- Base = `preserved_probes/regex_perf_probe_neutrality_948cbd63`
  (`948cbd63…`, cmp-verified against the pre-change release probe, SHA banked
  pre-change). Candidate probe preserved
  `preserved_probes/regex_perf_probe_barediag_fba8d1df` (`fba8d1df…`).
- Regex artifact: pre `1a5f7018…` → post (see `artifacts_post_regen.sha256`);
  all-11 regen train green incl. the ebnf fixed point; expected-delta review
  `overall=0` over all 11 (`artifact_delta_review.txt`) — every hunk is the
  `-0201` surfaces only.
- Battery (all green, `battery_summary.txt`): dual-feature lib **1000/0
  (29 ignored)** incl. the ALL-11 interpreter↔generated oracle + the new
  boundary test; cert ×3 seeds byte-exact `268/9/259/0 fully_certified`
  (spf=0); shape, duality, PCRE2 compile oracle, clippy source-strict.
- Every heavy step memory-guarded (16384 MB, floor 10%), serialized,
  `caffeinate -i`.

## Next

Per the one-fix-per-fresh-session directive this session stops after the
clean commit. NEXT (brand-new session) = `PGEN-RGX-0078-0202` (cascade error
design), then `-0203` (carrier core after its joint overlap contract), per
the `-0197` ordered leaves.
