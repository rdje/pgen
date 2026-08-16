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
  - "my control asserts an exact value and still passed under a broken implementation — how"
  - "how do I prove a ground-truth arm can actually go RED"
  - "how should I choose the input fixture for a synthetic control"
  - "is mutation testing worth it for a shell or python check script"
tags: [instrument-honesty, ground-truth, verification, corpus, adjudication, controls, staleness, mutation-testing, fixtures]
date: 2026-08-16
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.23 (the 9 base-less `enum [N:M]` pins); docs/tasks/artifacts/sv_corpus_grad/enum_base_range/verify_pins.py (POSITIVE + planted-failure NEGATIVE-A + typed-base NEGATIVE-B) and verify_pins.txt; docs/tasks/artifacts/sv_corpus_grad/enum_base_range/sweep.py (the unconditional-conclusion fix) with sweep.txt -> sweep_after.txt; docs/tasks/artifacts/sv_corpus_grad/enum_base_range/advertised_invalidity_recount.py (predicate pinned in code, baseline SEARCHED not defaulted); rust/src/ebnf_envelope_differential.rs (the same planted-mutation control, LANG-CAPABILITY-AUDIT.10.6). ⭐ The FIXTURE half added 2026-08-16 by docs/tasks/CI-PARITY-GATE-ROT.md leaf .34: 3 of 6 new ground-truth arms asserted exact set equality and still could not fail, because their invented fixtures balanced their own quotes or placed the candidate off command position — caught only by docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_string_reader_probes.sh, which mutates the LIVE reader per arm (8 arms, transcript in gate_reachability_string_reader_probes.txt).
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

## ⛔⛔ The harder version: a control can be unfalsifiable because of its FIXTURE, not its predicate

Everything above assumes the control's *predicate* is the weak part. `CI-PARITY-GATE-ROT.34`
(2026-08-16) found the other half, and it is easier to miss because the predicate looks exact.

The task was to stop a reachability instrument reading a make target named inside a shell error
*message* as an invocation. The fix shipped six synthetic ground-truth arms — three RED (this text
must yield no target) and three GREEN (this text must still yield one) — each asserting an **exact
set equality**, `invoked_targets(body) == expected`. All six passed. Then each was replayed against
a deliberately mutated reader, and **three did not fire**:

| arm | its fixture | why the mutation could not break it |
|---|---|---|
| multi-line `'…'` program | `{a: "x"}, {b: "y"}` | the payload's double quotes happened to **balance**, so dropping single-quote tracking desynced nothing |
| heredoc body | `print("one \" quote")` | same — a balanced payload |
| heredoc payload is data | `repair it with: make …` | the make text was not at **command position**, so it was never a candidate edge either way |

Each arm named the right mechanism and pinned the right verdict. Each was still a formality,
because the *input* could not exercise the mechanism. ⇒ **an exact assertion over an inert fixture
is a tautology with better manners** — and here it was about to ship inside the fix for a doctrine
whose entire subject is checks that cannot fail.

**The remedy is to make the mutation part of the arm's definition, not a later review step.** For
every control, write down the one-line change to the implementation that must break it, and run it:

```bash
# each arm names the edit that must turn it RED; the harness applies it to a COPY of the LIVE file
sed 's|data = string_data_lines(text) if shell_syntax else set()|data = set()|'   # → RED-1
sed 's|^            if c == "'"'"'":$|            if False:|'                     # → RED-2
```

Two properties make this cheap enough to do every time:

1. ⭐ **Mutate the live source, never a re-typed copy.** The harness `sed`s the real script so a
   future change to the reader is what gets tested; a hand-copied rule under test is its own
   documented failure (`GENERATED-LINT-CORRECTNESS.4`).
2. ⭐ **Fixtures come from the corpus, not from imagination.** The three inert fixtures were
   invented; their replacements were lifted from real gate scripts (`gsub(/"/, "")` — an **odd**
   quote count; a prose heredoc with one stray `"`), which is exactly why they discriminate. A
   census of the real inputs is what tells you which shapes are load-bearing.

⇒ the question to ask of any control is not *"is this assertion correct?"* but **"what edit to the
code would make this arm fail — and have I run it?"** If no such edit exists, the arm is
documentation. Related: [[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]]
(a control read off the emitting source is still a string you typed) and
[[a-negative-control-can-disable-the-assertion-it-is-testing]].
