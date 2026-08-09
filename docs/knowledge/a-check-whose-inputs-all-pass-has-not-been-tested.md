---
id: a-check-whose-inputs-all-pass-has-not-been-tested
title: When every input you have gives the same verdict, the control must be MANUFACTURED — plant a failure, or your verification is a formality
answers:
  - "how do I know my verification pass would actually fail if the thing were wrong"
  - "my per-file justification says 9/9 PASS — is that evidence or a tautology"
  - "what ground-truth control does a checker need when all its real inputs pass"
  - "how do I test the refusing branch of a gate that never refuses in practice"
  - "I am reclassifying corpus rows as must_reject — how do I avoid masking a real defect"
  - "is it safe to pin a corpus file's expected verdict on one construct when it might fail for another reason"
  - "why did my instrument keep reporting a finding that had already been fixed"
  - "a tracked number in a task leaf cannot be reproduced — what went wrong"
  - "what should the default baseline revision be for a before/after comparison"
  - "my before/after guard defaults to HEAD — why does it stop working after I commit"
tags: [instrument-honesty, ground-truth, verification, corpus, adjudication, controls, staleness]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.23 (the 9 base-less `enum [N:M]` pins); docs/tasks/artifacts/sv_corpus_grad/enum_base_range/verify_pins.py (POSITIVE + planted-failure NEGATIVE-A + typed-base NEGATIVE-B) and verify_pins.txt; docs/tasks/artifacts/sv_corpus_grad/enum_base_range/sweep.py (the unconditional-conclusion fix) with sweep.txt -> sweep_after.txt; docs/tasks/artifacts/sv_corpus_grad/enum_base_range/advertised_invalidity_recount.py (predicate pinned in code, baseline SEARCHED not defaulted); rust/src/ebnf_envelope_differential.rs (the same planted-mutation control, LANG-CAPABILITY-AUDIT.10.6)
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/enum_base_range/verify_pins.py | grep -q 'NEGATIVE-A e1 + planted early error -> EARLIER' && echo PLANTED-CONTROL-LIVE"
---

**A checker whose only observed outcome is PASS has not been tested; it has been run.**

The shape recurs whenever you justify a *reclassification*: N corpus rows all fail, you believe
they fail for one shared reason, and you want to pin their expected verdict on that reason. The
natural check — "confirm each row is stuck at construct X" — is a tautology if written naively.
You already know all N reject; that is why they were in the defect population. Any predicate loose
enough to match a stuck point *somewhere near* X returns N/N and tells you nothing. And the thing
it fails to tell you is exactly the dangerous one: **a file that rejects for a second, unrelated
reason gets a correct-looking pin, and the real defect behind it becomes invisible forever.**

⭐ **The fix costs ~15 lines: manufacture the failing input.** Take a known-good case, plant a
defect *before* the construct, and require the checker to classify it as stuck EARLIER. That
single control is the whole difference between a justification and a formality — it is the only
leg that exercises the refusing branch, and the refusing branch is what the entire argument rests
on. Measured on `SV-CORPUS-GRAD.3.23`: three controls guard nine pins, and only the planted one
proves the instrument can still say NO.

| control | what it proves | what its absence would allow |
|---|---|---|
| POSITIVE — the tracked reproducer classifies AT-CONSTRUCT | the anchor/span logic works | a detector that matches nothing, silently passing 0/N as N/N |
| NEGATIVE-A — a **planted earlier defect** classifies EARLIER | the checker can still REFUSE | rubber-stamping every row, including ones hiding a second defect |
| NEGATIVE-B — a legal near-miss yields ZERO matches | the construct detector discriminates | "the population" meaning every file containing the keyword |

The repo had already learned this one level up: the frontend↔meta-parser envelope differential
plants a mutation the differ must catch **exactly once, at exactly that index**, and aborts before
publishing a number if it does not. Same shape, different surface. When the expected verdict is
uniform across every input you have, ground truth cannot be *found* — it must be *built*.

⛔ **Three adjacent traps, all found in one leaf with zero parser bytes:**

1. **An instrument that can only narrate the problem expires when the problem is fixed.**
   `sweep.py` ended with an unconditional *"this is the mis-adjudication"* print. Re-run after the
   fix it reported `0 actionable` and then asserted the mis-adjudication anyway. Nothing consumed
   that line — but it writes a *tracked* artifact, and a tracked artifact stating a stale finding
   as a live one is how a repository accumulates confident wrong facts. Give every diagnosis
   instrument a resolved state.
2. **A tracked number whose predicate lives only in prose is a claim, not a measurement.** A leaf
   recorded "71 rows with a `_bad` basename". `_bad` as a *stem suffix* yields 64; `_bad` anywhere
   in the basename yields 71. Both readings are defensible from the prose, so re-deriving the
   number was a coin flip between measuring the recorded class and measuring a different one. Pin
   the predicate in code, behind a control that REFUSES unless it still reproduces the recorded
   figure.
3. **`HEAD` is not the pre-change baseline — it is the pre-change baseline only until you commit.**
   A before/after guard defaulting to `HEAD` is correct exactly while the change is unstaged, then
   silently *inverts*: it compares the new state against itself and passes forever. Search for the
   baseline by content (the newest tracked vintage reproducing the recorded number, named in the
   output) or refuse. Same class as the burn-down PICK step that quietly read a July vintage
   ([[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]]).

⚠️ And when the reclassification lands, report it for what it is. Rows leaving a defect population
because their *expectation* was corrected is not parser progress
([[a-rising-pass-rate-is-not-evidence-of-correctness]]), and it carries a cost worth stating: a row
pinned `must_reject` can never again testify about anything after its stuck point.
