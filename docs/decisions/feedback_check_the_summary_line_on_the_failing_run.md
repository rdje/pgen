---
name: feedback_check_the_summary_line_on_the_failing_run
description: "DISCIPLINE (2026-08-25, ENGINE-UNIVERSAL-SERVICES.46 slice 1) — an instrument's SUMMARY LINE is a claim, and it is the one claim a green run structurally cannot check. Measured: a probe's closing line counted EVALUATIONS and printed them as AGREEMENTS (`interpreter rung agreed on 8/8`) on the very run that had just detected two rung splits and failed. On every passing run the two counters are equal, so no amount of re-reading a green run could expose it — the flattering reading exists ONLY in the failing case. ⇒ when you drive a probe RED to prove it can fail, read its WHOLE output, not just the assertion message: the failure path is the only place a summary can be caught lying, and it lies in the passing direction."
id: feedback_check_the_summary_line_on_the_failing_run
title: "A summary line is a claim — and only the FAILING run can check it"
date: 2026-08-25
evidence: "rust/tests/profile_gate_negative_lookahead_generated_parser.rs (the `rungs_agreed` counter and the comment naming why it is not `interpreter_evaluated`); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .46 slice 1, section 'A DEFECT IN MY OWN INSTRUMENT, FOUND BY DRIVING IT RED'"
reverify: "make -C rust SHELL=/bin/bash profile_gate_monotonicity_gate 2>&1 | grep -q 'agreed on 8/8 evaluated' && echo SUMMARY-COUNTS-AGREEMENTS   # the word `evaluated` is the denominator being named; the defect was reporting the denominator as the numerator"
answers:
  - "my probe passes and its controls fire — what is left unchecked"
  - "why did driving a test red find a bug in the test itself"
  - "how do I check an instrument's reported summary is honest"
  - "what part of a passing run can never be validated by reading it"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

An instrument's closing summary — `N/N passed`, `agreed on 8/8`, `0 findings` — is a **claim about
the run**, and it is subject to the same standard as any other published number. But it has a
property no other assertion in the instrument has:

> **On a passing run, a wrong summary and a right summary print the same thing.**

So the summary is the one claim that a green run structurally cannot check. It can only be checked
where its inputs disagree — on the **failing** run. ⇒ when you drive a probe RED to prove it can
fail (which this repository already requires — [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]]),
**read the whole output, not just the assertion message.**

## What was measured

`ENGINE-UNIVERSAL-SERVICES.46` slice 1 built an 8-case probe that measures a verdict on two engines
and asserts they agree. To prove the rung-split detector worked, the interpreter side was forced to
request the wrong profile. It behaved correctly: two rows printed `⛔ NO` in the agree column, the
test failed, and the panic named both split cases.

And then it printed:

```
RESULT: 8 cases on the GENERATED parser; interpreter rung agreed on 8/8.
```

The counter incremented once per interpreter verdict *obtained* and was reported as verdicts that
*agreed*. On a green run those are the same number by construction. The line was correct in every
green run it had ever produced, and false in the first red one.

## Why the usual safeguards do not catch it

The existing disciplines all point at the verdict:

- declared expectations per case — checked the verdicts, which were right;
- a RED control on the assertion — fired correctly, and its message was accurate;
- a rung-split detector — fired correctly, and named both rows.

Every one of them passed *and* the summary was still wrong, because none of them reads the summary.
A summary is not an assertion; it is **narration**, and narration is where a green-biased reading
survives. Same family as the standing failure where a reader that treats what it cannot read as *no
findings* fails in the passing direction ([[feedback_enumerating_instrument_must_refuse]]) — here an
instrument that treats *measured* as *agreed*.

## The fix, and the shape of it

Count the thing you are naming. `rungs_agreed` is incremented only on agreement; the denominator is
printed as `evaluated` so the two numbers are visibly different quantities:

```
RESULT: 8 cases on the GENERATED parser; interpreter rung agreed on 8/8 evaluated.
```

⭐ The comment beside the counter says *why* it is not the obvious spelling, because the obvious
spelling is what a later author tidying the code would reach for. A guard whose reason is not written
next to it is a guard with a half-life.

## Related

- [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] — prove the needle
  can move. This record is what to do *while* you are proving it.
- [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — the verdict-side sibling:
  a case that cannot discriminate. Here it is the *report* that cannot discriminate.
- [[feedback_verify_a_claim_three_ways_before_publishing_it]] — a summary line is a published claim
  and inherits the whole standard.
