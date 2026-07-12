# RGX-0078 `.5.e` GLL REFUTED as a speed rung; the remaining destination gap is ARCHITECTURAL (measured ≈99× vs the `<5×` bound)

- **Type:** project
- **Date:** 2026-07-12 (session #104, `PGEN-RGX-0078-0034`, docs-only design spike)
- **Owner tree:** `docs/tasks/RGX-0078.md` (leaf `.5.e` — the full spike with all tool commands + the literature survey)

## Context

The agreed RGX-0078 speed road named `.5.e` (GLL + graph-structured stack + SPPF, research-grade) as
the rung after `.5.g`. The leaf's own mandate: survey the literature FIRST, then a worked mapping
([[feedback_research_grounded_sota_no_trial_and_revert]]). Five landed levers had brought the regex
8-pattern geomean 496µs → 56.8µs (8.76×).

## Decision / facts (tool-backed, re-runnable)

1. **GLL is REFUTED as a PGEN speed lever** (premise falsified BEFORE building — the `.5.d.3`
   adjudication class; nothing built, nothing reverted).
   - Literature: ALL(*) [Parr/Harwell/Fisher, OOPSLA 2014] — adaptive top-down prediction with DFA
     caching — outperforms GLL/GLR **by orders of magnitude** on real, near-deterministic grammars;
     even the strongest practical GLL [Afroozeh & Izmaylova, CC 2015] is on the losing side.
   - Mapping: post-`.5.c.2` PGEN is ALREADY a memoized near-deterministic RD parser with first-set
     pruning + delta-based winner-only effects. GLL would add descriptor/GSS/SPPF bookkeeping to
     EVERY rule entry to remove a backtrack residue RE-PROFILE #4 pins at **~2.2%**. Predicted
     net-NEGATIVE ⇒ unlandable under the ⛔ HARD CONSTRAINT (land iff faster + byte-identical).
2. **The distance to the `.6` destination is measured for the first time: geomean(PGEN/PCRE2) ≈ 99×**
   (8-pattern bench preview; PGEN 56.6µs vs PCRE2 10.47 `pcre2test -t` ≈0.57µs, same machine). The
   `<5×` bound needs a further **≈20×**, while ALL remaining profiled buckets sum to ~32% (≈1.5× if
   fully eliminated). **No bucket lever reaches the destination.**
3. **WHERE the 20× lives:** the rule-cascade-per-char execution model — ~45 rule entries to parse a
   4-char pattern (≈11/char × ≈262ns/entry, memo-stats-counted); PCRE2 compiles the whole pattern for
   the cost of ~ONE PGEN rule entry. Addressable only by executing FEWER rule entries (cache /
   fusion / scanner compilation), not by shaving buckets.

## Consequences

- `.5.e` is closed as refuted; the road-level steering decision was surfaced to the director
  (options: **`.7` parse-cache+batch (recommended)** — workload-honest, cache hit ≈0 amortized;
  **cascade-fusion / grammar-derived scanner codegen** — the honest research-grade replacement,
  ceiling ~3–5× via entries/char ~11→~2, STEP-0 fusibility census first; **re-baseline the `.6`
  bound** — raw `<5×` vs a no-AST single-pass C compiler may be the wrong contract for a
  grammar-driven parser emitting a typed JSON AST).
- Either way `.6` needs `pgen_iteration_flow` vendored — the official corpus metric does not exist
  yet; the 99× is an 8-pattern preview.
- Standing lesson reinforced: measure the marginal premise BEFORE building the high-risk change
  (`.5.d.2`/`.5.d.3`/`.5.e` now all confirm it) — the research-grounded doctrine turned a
  research-grade rewrite into a zero-cost adjudication.
