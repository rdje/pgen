---
name: project-rgx-0078-sub-1us-closure-target
description: Director 2026-07-13 (session #110) — RGX-0078 closure bar RE-BASELINED to an ABSOLUTE target; regex COLD parse geomean ~<=1us with the EBNF (+annotations) remaining the SOLE source of truth; hit it => campaign closes, focus pivots to other parsers. Supersedes the relative <5x-vs-PCRE2 bound.
metadata:
  node_type: memory
  type: project
---

**Director directive (2026-07-13, session #110, verbatim):** *"Look, if we can get just sub-1us
or at 1us with EBNF (+annotations) source of truth, I can stop there and focus on other
parsers."* (Context immediately prior: *"not necessarily 0.57us. Just below 1us runtime would
be lit."* — the target is decoupled from PCRE2's exact number.)

**The re-baselined closure bar for `RGX-0078.6`:**
- regex **COLD parse geomean ≈≤1µs** (the 8-pattern bench statistic; the official corpus
  metric still runs over the PCRE2 corpus once `pgen_iteration_flow` is vendored);
- with `grammars/regex.ebnf` + annotations remaining the **sole semantic source of truth**
  ([[project-ebnf-is-single-source-of-truth]] + the §H division-of-labor contract: the spec
  captures FUNCTIONALITY, the optimizing compiler MANUFACTURES the speed — no hand-written
  parser escape hatch, no speed vocabulary leaking into the EBNF);
- accuracy floor unchanged and immovable ([[feedback-correctness-before-speed]]): every
  increment lands only measured-faster + byte-identical under the full oracle battery.

**What it supersedes:** the prior RELATIVE closure bound `geomean(PGEN/PCRE2) < 5×`
(≈<2.85µs at PCRE2 ≈0.57µs) from [[project-rgx-0078-regex-slowness-followup]]. The new bar is
ABSOLUTE and tighter (≈1.75× PCRE2 on the current bench numbers).

**Road consequence (recorded, not a new road):** the planner rung continues exactly as agreed
(P1 cascade/wrapper inlining → P3 selective machinery → P4 value folding; census-fixed order),
composing to a projected ≈15–20µs. The DEEP-SPECIALIZATION planner extensions — byte-switch
subset dispatch at overlap sites, full leaf-cascade folding toward ~1–2 rule entries/char,
memoization only where backtracking is provably real, direct value construction ("emit what a
hand-written parser would be" wherever analysis proves it invisible) — are hereby ON-ROAD after
P1/P3/P4, iterating until the ≤1µs bar is met. `.7` batch + parse-cache remains a workload
BONUS (amortized ≈0 on repeats), NOT the closure criterion: closure is the COLD number.

**Feasibility anchor (census arithmetic, honest):** ≤1µs on a ~30-char pattern ≈ ~30ns/char
budget ⇒ BOTH multiplicative levers must land: entries/char ~11 → ~1–2 AND per-entry cost →
raw structural work (today ≈40–55ns). The measured always-on observability tax (V4 ≈3.4%) is
NOT the blocker; entry count × entry cost is. Every landed pass so far has hit its census
prediction (P0: −25.8% vs ≈−25% predicted; P2: −5.3% vs −3–7% predicted).

Status snapshot at the directive: geomean ≈39.7µs (496µs → 39.7µs = 12.5×, seven levers,
all byte-identical); next slice = P1.
