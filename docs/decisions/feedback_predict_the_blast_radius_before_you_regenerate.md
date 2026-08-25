---
name: feedback_predict_the_blast_radius_before_you_regenerate
description: "DISCIPLINE (2026-08-25, ENGINE-UNIVERSAL-SERVICES.46 slice 3) — WHICH FILES CHANGED is a correctness instrument, and it is the one that fires when no test does. MEASURED: a codegen repair predicted to move 1 of 11 generated artifacts moved ALL ELEVEN; gating the emission still moved a second family the same session's own census had reported CLEAN; chasing that second discrepancy is what surfaced a SEMANTIC design error — the repair would have made SystemVerilog's dialect-selecting keyword guard union both dialects' reserved words, so `reg class;` (legal IEEE 1364-2005) would have stopped parsing under verilog_2005. A rejection introduced by a fix for an over-acceptance, and no test failed at any point. ⇒ write the predicted blast radius DOWN before regenerating, so the gap has somewhere to show up; when a change moves more than predicted, the gap is a FINDING, not a chore."
id: feedback_predict_the_blast_radius_before_you_regenerate
title: "Which files changed is a correctness instrument — predict the blast radius before you regenerate"
date: 2026-08-25
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .46 slice 3, the three-row blast-radius table (blanket 11/11 -> reachability 2 -> satisfiability 1); docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/selection_vs_narrowing_probe.sh (the two-sided pin on the corrected design, run against grammars/systemverilog.ebnf)"
reverify: "bash docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/selection_vs_narrowing_probe.sh | grep -q 'pass=5 fail=0' && echo SELECTION-CASE-INTACT   # arm 1 is the row the over-applied repair would have broken"
answers:
  - "my change compiles and every test passes — what have I not checked"
  - "a regeneration moved more files than I expected, does it matter"
  - "how do I catch a semantic error that no test covers"
  - "what should I write down before running a codegen change through regeneration"
  - "why did tightening an emission condition find a design bug"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

Before regenerating after an engine change, **write down which artifacts you expect to move, and
why.** Then regenerate and compare.

> **When a change moves more than you predicted, the gap is a finding — not a chore.**

The prediction is what gives the difference somewhere to show up. Without it, "everything
regenerated" and "exactly the right thing regenerated" produce the same feeling.

## What was measured

`ENGINE-UNIVERSAL-SERVICES.46` slice 3 repaired a profile-gate soundness inversion in codegen and the
interpreter. The per-site emission was conditional, so the prediction was: **1 of 11 generated
artifacts moves.**

| version | artifacts moved | what the gap revealed |
|---|---|---|
| first cut | **11 of 11** | the machinery was emitted into the shared skeleton every parser gets — including a branch in `memoized_call`, the hottest emitted function, for grammars that could never take it |
| emission gated on need | **2** (`regex`, `systemverilog`) | regex was moving for a site the same session's own census had reported **clean at 0** |
| condition corrected | **1** (`systemverilog`) | regex returned to its pre-repair hash |

Chasing the *second* row is what found the real defect. The regex site forced a transitive census,
which raised SystemVerilog's population from 3 to 6 and put a rule on screen that had not been
looked at: `non_keyword_identifier`, whose guard is
`!reserved_non_keyword_identifier` over a dispatcher of two per-dialect sibling rules. The repair as
designed would have made that guard see **both** dialects' keyword lists at once — so `reg class;`,
legal IEEE 1364-2005, would have stopped parsing under `verilog_2005`.

**A rejection introduced by a fix for an over-acceptance, in a family under a direct director order,
and no test failed at any point.**

## Why the usual instruments cannot do this job

Every gate in the repository asks whether the artifacts still behave. They were all green: the
corpus ratchet had nothing to compare against yet, the unit banks tested the new detector's own
logic, and the defect pin was measuring the synthetic case the repair *did* fix. A semantic collision
between a repair and an idiom in one real grammar is invisible to all of them until something makes
you read that grammar.

The blast radius is what makes you read it. It is cheap (a hash per artifact), it has no false
negatives about *change*, and it points at a file rather than at a symptom.

## The corollary that costs money

**Shared-skeleton emission spreads a local change globally by default.** A code generator that emits
one parser skeleton will put your new field, your new helper and your new hot-path branch into every
family unless you gate them. The blast radius is the measurement that says so — and in a project
whose second non-negotiable is peak speed, a cost with no benefit is rejected, not traded.

## Related

- [[feedback_ask_the_instrument_the_question_its_founding_case_cannot_answer]] — the sibling
  discipline: a new detector is fitted to its founding case, so test it on the differently-shaped one.
- [[feedback_check_the_summary_line_on_the_failing_run]] — the report-side sibling.
- [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — why the corrected design
  got its own two-sided pin, on the real grammar rather than a synthetic.
