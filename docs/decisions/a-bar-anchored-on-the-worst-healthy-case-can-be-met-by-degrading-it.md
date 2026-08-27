---
name: a-bar-anchored-on-the-worst-healthy-case-can-be-met-by-degrading-it
description: "DISCIPLINE (2026-08-28, ENGINE-UNIVERSAL-SERVICES.44(d) third ruling / PGEN-ENGINE-UNIVERSAL-SERVICES-0108) — a quality bar expressed as a RATIO against the subject's own worst acceptable case is satisfiable by making that case worse, and it will be. MEASURED: the bar 'an over-ceiling rejection may cost no more than 2x the parser's deepest ACCEPTED parse' was derived from the parser rather than invented, cannot rot across hardware, and scored both released families RED — all of which was right. Then bisection found VHDL's real accept cliff at depth 681 (a ladder had stopped at 400) and the anchor turned out to cost 18.557 s: VHDL scores 18.735/18.977 = 1.0x and PASSES, from a parser needing nineteen seconds either way on 1.4 KB. The accept path had a 102x shoulder between depth 575 and 681 that no reject-path analysis could see. ⇒ anchor the ratio in the subject's HEALTHY regime, and add a second leg bounding the anchor itself; a bar a catastrophic subject passes is worth exactly as much as a bar every subject passes."
id: a-bar-anchored-on-the-worst-healthy-case-can-be-met-by-degrading-it
title: "A bar anchored on the subject's worst healthy case can be met by degrading it — bound the anchor too"
date: 2026-08-28
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (d) third ruling (the five-family two-leg table + the VHDL accept-path cost curve) and leaf .60; producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh arm E; log docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/layout_aware_dispatch_census.txt"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh   # arm E bisects each cliff and times min-of-five; the vhdl row must keep reading ~1.0x on leg 1 with an accept cost in the tens of seconds — that PASS beside that cost is the whole record. If vhdl's accept cost ever falls below a second, .60 has been fixed and this record's counterexample is gone."
answers:
  - "how do I set a quality bar without inventing a number"
  - "my ratio bar passes, is that good"
  - "why did a bar that both subjects failed suddenly pass for one of them"
  - "what anchor should a performance ratio use"
  - "how can a self-derived threshold be gamed without anyone gaming it"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-28
---

## The rule

When you derive a bar from the subject instead of inventing one — a good instinct, recorded
separately as [[derive-the-bar-from-the-subject-when-no-authority-owns-the-number]] — check what
happens to the bar when the **anchor** degrades. If the anchor is the subject's *worst acceptable*
case, the ratio can be met by making that case worse, and no adversary is required: a defect
elsewhere in the system does it for you.

Anchor in the **healthy** régime, and add a leg that bounds the anchor itself.

## Why — the measurement

`ENGINE-UNIVERSAL-SERVICES.44`(d) ruled: *an over-ceiling rejection may cost no more than 2× the same
parser's **deepest ACCEPTED** parse.* Everything about that was defensible — derived rather than
invented, host-independent, one-sentence justification (*a parser must not spend more effort failing
than succeeding*), and it scored both released families RED, which is the test of whether a bar is
worth having.

Then the anchor was measured properly. A hand-written ladder had put VHDL's accept cliff at depth
400; bisection put it at **681**. And the accept path at 681 costs **18.557 s**:

| depth | 300 | 500 | 575 | 600 | 625 | 650 | 675 | **681** (cliff) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| verdict | accept | accept | accept | accept | accept | accept | accept | **accept** |
| cost | 0.076 s | 0.131 s | 0.182 s | **0.931 s** | **4.516 s** | **10.277 s** | **17.034 s** | **18.557 s** |

So VHDL scores `18.735 / 18.977` = **1.0×** and **passes** — a parser that needs nineteen seconds
either way on 1.4 KB of input. The sibling family, SystemVerilog, is flat across its whole accept
range (0.040 s → 0.182 s) and fails at ≈70–90×. **The two families are broken in different ways and
the single-leg bar could only express one of them.**

## The repair

Two legs, both derived from the parser, both required:

1. **Ratio leg** — a rejection may cost no more than 2× an accepted parse of the same shape, anchored
   at the deepest depth whose accept cost is still within 2× of a shallow reference accept.
2. **Anchor leg** — no accepted depth may cost more than 2× the accepted depth at 80 % of the cliff.

Re-scored: SystemVerilog RED on leg 1, green on leg 2. VHDL green on leg 1, **RED on leg 2 at 102×**.
Both still fail, which is what the original ruling's own test demanded — and now the two failures are
distinguishable.

## The shape to reuse

- Write down what the bar's **denominator** is allowed to be. If nothing bounds it, the bar is
  conditional on a quantity you are not watching.
- Ask the degradation question explicitly: *if the anchor got ten times worse, would this bar notice
  or would it improve?* If it improves, the bar is inverted somewhere.
- A ratio hides absolute magnitude by design. That is its strength across hardware and its blind spot
  within one machine, so pair it with something that sees magnitude.
- ⚠️ NOT the lesson: ratios are bad. The ratio leg survived the challenge intact and is still the
  part that cannot rot. What failed was choosing the most extreme legal anchor for it.
