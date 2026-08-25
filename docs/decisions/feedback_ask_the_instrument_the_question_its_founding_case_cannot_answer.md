---
name: feedback_ask_the_instrument_the_question_its_founding_case_cannot_answer
description: "DISCIPLINE (2026-08-25, ENGINE-UNIVERSAL-SERVICES.46 slice 2) — a new instrument is FITTED to the case that motivated it, so that case is the one case that cannot validate it. MEASURED: a lookahead detector built from the SystemVerilog finding reproduced the hand count there EXACTLY (3/3) and reported 0 for grammars/regex.ebnf — a grammar with 95 `!` occurrences — because TWO defects in the shared profile plumbing (context read from the FILTERED grammar, whose load-time filter had already deleted the gated rules AND their annotations; and a profile universe built only from `@profiles` lists, which name where a rule is PRESENT and never where it is ABSENT) are both INVISIBLE on SystemVerilog, which declares no `@default_profile`. ⇒ before trusting a new detector, run it on the population member with the MOST DIFFERENT SHAPE, not the one it was built from — and encode that case as a control the census refuses to run without."
id: feedback_ask_the_instrument_the_question_its_founding_case_cannot_answer
title: "The founding case is the one case that cannot validate your instrument"
date: 2026-08-25
evidence: "docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/census.sh (the SELF-CHECK arm and its refusal) + default_profile_universe_control.ebnf (the five-rule reduction of grammars/regex.ebnf's shape); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .46 slice 2, section 'SIZING FOUND TWO DEFECTS IN THE INSTRUMENT'; the routed consequence is leaf .47"
reverify: "bash docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/census.sh | grep -q 'SELF-CHECK: the @default_profile control reports 1 finding' && echo FOUNDING-CASE-CONTROL-HOLDS   # the census REFUSES to print a population if the control finds nothing, so a zero is a measurement rather than a blind spot"
answers:
  - "my new detector reproduces the case I built it from — what have I actually proven"
  - "how do I choose which case to test a new instrument against"
  - "why did a detector report zero on a grammar that obviously has the defect"
  - "what makes a census's zero trustworthy"
  - "an analysis reads a structure some earlier pipeline stage produced — what can go wrong"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

A detector written from a finding is **fitted to that finding**. Reproducing it proves the fit, not
the instrument. Before quoting a population:

> **Run it on the population member whose SHAPE differs most from the founding case, and make that
> case a control the census refuses to proceed without.**

The founding case cannot do this job, because every assumption you made silently is an assumption
that case satisfies.

## What was measured

`ENGINE-UNIVERSAL-SERVICES.46` slice 2 added a lint arm for lookaheads whose subject `@profiles`
gates out of a profile the referring rule is still live in. On `grammars/systemverilog.ebnf` — the
grammar the finding came from — it reproduced the hand-sized population exactly: **3 negatives, the
three hand-named `!scope_resolution` sites**.

It also reported **0** for `grammars/regex.ebnf`, a grammar with **95** `!` occurrences and seven
`@profiles`-gated rules. That zero was false, for two independent reasons, **neither of which can
occur on SystemVerilog**:

1. **The lint's profile context came from the FILTERED grammar.** A grammar declaring
   `@default_profile` has its gated rules removed by the load-time filter — and their `@profiles`
   annotations with them. Measured: the lint printed `profiles=[]`, an *empty* universe, on a
   five-rule grammar whose runtime verdicts show the defect in both directions.
2. **The profile universe omitted `@default_profile`.** It is the union of `@profiles` lists, and a
   gated rule's list names the profiles it is *present* in. The profile it is *absent* from — the
   only place the defect exists — is by construction in no list. For regex that is `pcre2`, the
   profile every parse uses unless one is requested.

SystemVerilog declares no `@default_profile`. Both defects are invisible there, in the passing
direction, forever.

## Two generalisations worth carrying

- **An analysis that reasons ACROSS configurations must not read a structure produced by choosing
  one.** Look for this wherever a pipeline narrows before a checker widens: the narrowing stage is
  entitled to delete exactly what the checker needs.
- **A set derived from positive declarations cannot name its own complement.** When something is
  declared by listing where it applies, absence-shaped defects live in what nothing declares — so ask
  what the complement is and whether any directive names it.

## The fix is one control, and it runs FIRST

`census.sh` runs a five-rule reduction of regex's shape — `@default_profile: pcre2` plus a rule gated
to `["relaxed"]`, so the only route to its defect is a profile appearing in no `@profiles` list — and
**refuses to report a population at all** if that control finds nothing. Without it, *"regex reports
0"* and *"the instrument never tested regex's shipping profile"* are the same output.

## Related

- [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] — prove the needle
  can move. This record says **on which input**: the one the instrument was not built from.
- [[feedback_check_the_summary_line_on_the_failing_run]] — the same failure class one level up, in
  the report rather than the detector.
- [[feedback_instrument_needs_ground_truth]] — ground truth constrains the verdict; this constrains
  the *population* the verdict is quoted over.
