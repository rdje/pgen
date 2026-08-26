---
name: name-the-success-case-or-it-is-counted-as-the-symptom
description: "DISCIPLINE (2026-08-27, LIVE-DOC-CONTAINMENT.9 / PGEN-LIVE-DOC-CONTAINMENT-0014) — a detector built from *the symptom is visible in the output* must ALSO enumerate what CORRECT output looks like, or the success case is counted as the symptom. MEASURED on a contract this project ships: the rule *a backtick that survives into the rendered text was not consumed as a delimiter, so the source is imbalanced* is renderer-authoritative and sounds airtight — and it over-counted by 31 %, reporting 21 defective cells across 12 rows against a true 16 across 11. A backtick rendered INSIDE a `<code>` element is the author's correctly-escaped content: it is the double-backtick idiom WORKING. One row (`SV-0046`) was entirely healthy and was routed for repair; another (`SV-0048`) scored 5 cells against a real 1. ⛔ The over-count is invisible from the detector's own side, because every reported cell really does contain a backtick — the reading is not wrong, the INTERPRETATION is, and no amount of re-running changes it. ⇒ before a detector's headline becomes a defect count, write down the shape of a CORRECT artifact that the detector would also flag, and exclude it explicitly; a symptom-based rule inherits the ambiguity of its symptom."
id: name-the-success-case-or-it-is-counted-as-the-symptom
title: "Name the success case, or the detector counts it as the symptom"
date: 2026-08-27
evidence: "docs/tasks/LIVE-DOC-CONTAINMENT.md leaf .9, field `RETRACTION_of_my_own_routing_census_2026-08-27` (21/12 retracted to 16/11, with SV-0046 healthy and SV-0048 at 5-against-1) against the routing census in the same leaf's `Routing_evidence`; the corrected arm and its comment in docs/tasks/artifacts/live_means_live/table_render_oracle.sh (function `backtick_census`)"
reverify: "bash docs/tasks/artifacts/live_means_live/backtick_detector_fixture.sh   # two rendered cells, one healthy one broken: the loose detector totals 2, the <code>-excluding one totals 1; exits non-zero if that stops holding"
answers:
  - "my detector reports more defects than actually exist, why"
  - "how do I validate a detector built from a visible symptom"
  - "why did re-running my census not catch the over-count"
  - "is a renderer-authoritative check automatically correct"
  - "what should I check before quoting a detector's count as a defect count"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A detector defined as *"the symptom is visible in the output"* is only half a definition. Write the
other half: **what does a CORRECT artifact look like, and would this detector flag it too?** Exclude
that shape explicitly. Until you have, the detector's headline is a count of *occurrences*, not a
count of *defects*, and the two are different numbers.

## Why — the measurement

The defect was real: a single-backtick Markdown code span cannot contain a backtick, so quoting a
SystemVerilog compiler directive (`` `timescale ``, `` `include ``) inside one breaks the span and
the leading backtick — the part that makes it SystemVerilog — falls out of code formatting.

The detector was renderer-authoritative, which felt like the strong move: *a backtick that survives
into the rendered text was not consumed as a delimiter, so the source is imbalanced*. It needs no
model of backtick runs, no pairing, nothing that can itself be wrong. It reported **21 cells across
12 rows**.

The correct answer is **16 across 11**. The double-backtick idiom — `` `` `ifdef `` `` — renders as
`<code>`ifdef</code>`, and that backtick is **content**. It is the fix, working, rendered exactly as
the author intended. `SV-0046` had no defect at all and was routed for repair; `SV-0048` was reported
at 5 cells against a real 1.

## Why re-running could never catch it

Every cell the detector reported really did contain a backtick in its rendered text. The measurement
was accurate. What was wrong was the claim attached to it — that a backtick in the output implies an
imbalance in the source. A second run, a third renderer, a larger sample: all of them agree with the
first, because none of them disputes the reading. Only naming the success case disputes the
interpretation.

## The transferable shape

Watch for this wherever a check is phrased as *"X appears in the output"*:

- a lint that flags `TODO` in source — and flags the one in the lint's own test fixture;
- a leak detector that flags an allocation without a matching free — and flags the arena that frees
  in bulk;
- a "hardcoded secret" scan that flags the documented example key;
- any grep-shaped rule over generated output, where the generator legitimately emits the token.

The tell is that the false positives are **the mechanism working**, so they cluster exactly where
the practice is best, and a reviewer sampling the report sees plausible hits everywhere.

⚠️ Note what is NOT the lesson: renderer-authoritative was the right instinct, and the corrected
detector still uses it. The defect was in stopping one step short of the definition.

Related: [[feedback_instrument_needs_ground_truth]] and
[[feedback_ask_the_instrument_the_question_its_founding_case_cannot_answer]] — the same family; this
one is specifically about the detector's **negative** class rather than its positive one. And
[[a-delimiter-imbalance-is-not-a-local-defect]], the defect this detector was built to find.
