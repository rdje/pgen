---
id: a-ratchet-ceiling-goes-stale-from-below-when-the-instrument-improves
title: A ratchet ceiling goes stale from BELOW when the instrument improves — attribute a ratchet's delta with the binary pinned, or the instrument's change is booked to the grammar
answers:
  - "why does the envelope ratchet fail with a count BELOW its ceiling"
  - "why did the ceiling vintage's grammar not reproduce its own ceiling under the current binary"
  - "how do I attribute a ratchet count's movement to grammar commits rather than to frontend changes"
  - "what is a pinned-binary per-vintage census and why is the delta not simply count minus ceiling"
  - "how were the 211 systemverilog envelope divergences adjudicated and why is the SV grammar not wrong at any of them"
  - "why can a divergence mask another divergence at the same site"
tags: [grammar-wellformed, ratchet, envelope-differential, two-vintages, measurement-discipline, ebnf-frontend]
date: 2026-09-24
status: current
evidence: "GRAMMAR-WELLFORMED.H.20.3 (PGEN-GRAMMAR-WELLFORMED-0185): under the HEAD binary the systemverilog ceiling vintage 958fcc24 reads 150, not its recorded 155 (H.20.1's return_scalar class gone with EBNF-FRONTEND-SILENT-TRUNCATION.6); return_annotation reads 5 < ceiling 6 at three grammar vintages alike; a 51-vintage census (22 movers, +83/-22 gross) sums to 211 exactly; control C5 removed one predicted pair and surfaced one masked row."
reverify: "grep -nE 'systemverilog\\) echo 211|return_annotation\\) echo 5' rust/scripts/ebnf_frontend_dual_run_diff_gate.sh; PGEN_ENVELOPE_DUMP_ALL=1 rust/target/debug/ebnf_dual_run_diff --input grammars/systemverilog.ebnf --output /tmp/o.json --envelope-differential /tmp/e.json && python3 -c \"import json;print(json.load(open('/tmp/e.json'))['envelope_differential']['divergence_total'])\""
---

**A ratchet count is two numbers with two provenances — the grammar the count was taken on and the
binary that took it. When the binary improves, the count falls, the ceiling stays, and the gate is
red from BELOW; when the grammar moves, only a census with the binary PINNED says which commits
moved which sites. `count − ceiling` is neither.**

## The two ways a ceiling goes stale

- **From above** — grammar commits add sites (a `@profiles:` line above a rule is one more
  instance of the meta-parser's leading-annotation rebinding, `.10.14`). The ceiling moves only by
  adjudicating every added row: named, attributed to its commit, classed as a known defect instance
  or argued as a legitimate frontend difference ([[GRAMMAR-WELLFORMED.H.20]]'s rule).
- **From below** — a frontend fix removes a whole class (`EBNF-FRONTEND-SILENT-TRUNCATION.6` removed
  `H.20.1`'s `return_scalar → return_object` rows) and nobody lowers the number. The ceiling vintage's
  own grammar no longer reproduces its ceiling (`958fcc24`: recorded 155, reads 150), and a second
  grammar sits red at `5 < 6` for weeks. A two-sided ratchet is what makes this a red row rather than
  a silent slack the next regression can hide in.

## The census, and why the binary is pinned

Run the differential at EVERY grammar vintage since the ceiling with ONE binary (HEAD's), diff the
divergence lists as multisets keyed on `(rule_name, comparison, arm1.kind, arm2.kind)`, and sum the
per-vintage deltas: if the sum lands on the measured count, every row has a commit. Vary the binary
instead and the instrument's improvements and regressions are booked to the grammar (the *one report,
two vintages* trap, `TOOLBOX.md` §1.3). Gross and net differ — sites removed and re-added on the way
count twice on the path and zero end-to-end — so both are stated.

## A divergence can mask another at the same site

Removing one leading annotation removes its pair (the `<absent> → semantic_annotation` on rule *i*
and the `semantic_annotation → <first body token>` on rule *i+1*) and lets the NEXT rule's leading
annotation rebind onto rule *i+1*, a new row where none showed before. So the post-fix count is not
`count − rows_fixed`; it is measured. Related:
[[a-fallback-arm-goes-inside-the-shared-rule-as-an-ordered-last-alternative]] (the cost ratchet as
the instrument), [[GRAMMAR-WELLFORMED.H.20]].
