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

---

**⭐ RE-AFFIRMED + ELEVATED TO EXISTENTIAL (director 2026-07-18, session #147, fork answer):**
presented with the `.5.i.16` strategic fork (micro-seam thinning — first lever with LOW below
the −2.0% land bar; micro-levers plateau ≈3.8–4.0µs; the construction/allocation cluster
≈45–50% = the only ≤1µs-scale seam), the director selected **(A) — keep the ≤1µs bar, open the
REPRESENTATION-ROAD STEP-0 scout**, with this rationale (verbatim, typo preserved): *"I select
(A), because if we can reach 1us or under, this project will is worthless in the eyes of many
developers. Achieveing 1us or under developers might take PGEN parsers seriously. So, it is a
live or die situation. At least that's the way I see it."* — i.e. WITHOUT ≤1µs the project is
worthless in many developers' eyes; AT ≤1µs developers may take PGEN parsers seriously; the
bar is **live-or-die for PGEN's credibility**, not a nice-to-have. Consequences: the
representation road (committed-value spans/arena representation + thin-memo replay-by-reference
+ doomed-construction elision) is now THE road to closure — tracked as the RGX-0078 `.5.j`
lane, STEP-0 scout first; the parked `.5.i.17` dispatch emission and the queued C4 census are
subordinate tail levers, to be run only when they compose toward the bar. The accuracy floor
remains immovable ([[feedback-correctness-before-speed]]).

---

**⭐ BAR REDEFINED — MAXIMUM, PCRE2-CORPUS-WIDE, WITH A STANDING HOLD GATE (director
2026-07-18, session #150, mid-session directive, verbatim):** *"When I ask to reach this
sub-1us regex runtime threshold, it means that maximum observable run-time shall be < 1us.
Meaning the slowing parse time shall less than 1us. this shall be achieved on PCRE2 external
test corpus. Once we reached that objective then I/we can move to other accuracy related
tasks. After that, whatever further change is made to the AST pipeline we need to ensure that
sub-1us holds."* + *"That sub-1us threshold applies only to PGEN regex parser, of cource."*

The four consequences (each a tracked obligation):
1. **The metric is the MAX, not the geomean** — the slowest parse over the corpus bounds the
   claim. The 8-pattern bench geomean stays the campaign's steering instrument, but closure is
   judged on the corpus-wide worst case.
2. **The population is the PCRE2 external test corpus** (`regex_corpus_bundle` canonical
   oracle cases; real tuple 2189/1879/262/48) — regex parser only.
3. **After closure ⇒ pivot to accuracy tasks**, and **every subsequent AST-pipeline change
   must PRESERVE sub-1µs** — a standing corpus max-time REGRESSION GATE joins the battery at
   closure (deliverable recorded in the RGX-0078 tree).
4. **The tracked catastrophic-backtracking corpus cells become BLOCKING pre-closure work**
   (sweep lines 878/881/1340 — today effectively non-terminating and excluded from sweeps per
   [[feedback_dont_run_jobs_that_hit_known_pathological_inputs]]): under a max-bound there is
   no exclusion — a hang IS the maximum.

**⭐ THE GOAL BEHIND THE BAR — PERCEPTION, MEANS DELEGATED (director 2026-07-18, session #151,
verbatim, typos preserved):** *"I do not want developers to discard PGEN regex parser because
they see it as slow. To the contrary, they should see it as as a very good alternative to PCRE2
because it is really fast. How to achieve this perception, is your call."* — The sub-1µs bar is
the PROXY; the OBJECTIVE is the developer-perception "fast, credible PCRE2 alternative", and the
operationalization is explicitly ENGINEER-OWNED.

**The engineer's operationalization (decided 2026-07-18 session #151, recorded here as the
standing reading of the bar):** developers form the speed perception from (a) the headline
comparison against the incumbent and (b) the absence of pathological outliers. Therefore the
PERCEPTION BAR = two measured conditions on the PCRE2 external corpus (later also rebar,
per the queued corpus-expansion record):
1. **TYPICAL-SPEED** — PGEN regex parse competitive with PCRE2's own compile on the same
   patterns; the recorded size-normalized reading (absolute <1µs for ≤p99-size cells) stays as
   the concrete embodiment (PCRE2 ≈0.57µs on bench-typical patterns ⇒ <1µs ≈ within ~1.75×).
2. **NO-OUTLIER** — a hard LINEAR tail bound corpus-wide (≤ ~8 ns/byte): no cell may look
   pathological in a published distribution. One 2ms cell does more perception damage than a
   20% geomean gap ⇒ the `.5.j.4` superlinearity class is PERCEPTION-CRITICAL work.
New measurement obligation queued: a per-cell **PCRE2 compile-time column** beside the PGEN
column in the corpus baseline (the exact comparison developers will make; geomean AND max
reporting per the standing doctrine). The road/battery/land-gate discipline is unchanged;
`.5.j.4`'s root-cause slice remains the feasibility VERDICT for condition 1's MAX form.

---

**Engineer's honest physics + single recommendation (surfaced 2026-07-18, proceeding per the
2026-07-15 fork-discipline amendment; director veto welcome):** the corpus length profile is
p50=10 B / p90=32 B / p99=121 B / max=3,511 B (2,189 cases). An ABSOLUTE max < 1µs including
the 3,511-byte pattern requires ~0.28 ns/byte sustained incl. AST construction — beyond any
AST-building parser (PCRE2's own compile of such patterns takes multiple µs). RECOMMENDATION:
read the directive as **max < 1µs for every corpus pattern of typical size (≤ the p99 ≈ 128
bytes — 99% of the corpus), with the >p99 tail (the ~22 largest patterns) bounded by the SAME
implied throughput (≤ ~8 ns/byte, i.e. time < len × 8ns)** — so the worst case is bounded
everywhere and the bar stays physically meaningful; plus the catastrophic-backtracking fixes
as blocking pre-closure leaves. The measurement deliverable: a corpus-wide per-pattern
parse-time distribution (max/percentiles vs length) as the new closure baseline, landed after
the in-flight `.5.j.2` STEP-2a emission (which reduces per-parse cost under any bar reading).
