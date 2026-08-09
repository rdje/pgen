---
id: prove-each-refusal-path-with-an-input-only-it-can-trigger
title: A refusal that fires for the WRONG reason is not evidence — prove each guard with an input only that guard can trigger, and read the exit code where the process exits
answers:
  - "how do I prove a new REFUSE path in a script actually works"
  - "my guard fired so is it verified"
  - "why did my refusal demo print exit=0 for every case"
  - "I am adding a row-count consistency check between two artifacts — what do I check first"
  - "how big should the diff of a bug fix be"
  - "is wc -l a safe way to count rows in a TSV"
tags: [instrument-honesty, ground-truth, proof-discipline, gates, shell, tsv]
date: 2026-08-09
status: current
evidence: docs/tasks/artifacts/sv_corpus_grad/families_classifier_vintage/vintage_and_corruption.txt (§4, all four paths with real exit codes, plus the two ways the first attempt was wrong); stimuli/sv/classify_rejects_valid_families.py (the malformed-line and row-count refusals + ground_truth_controls); stimuli/sv/cluster_rejects_valid.py (tsv_cell, and the narrowing note); docs/tasks/SV-CORPUS-GRAD.md leaf .3.21
reverify: "python3 stimuli/sv/classify_rejects_valid_families.py --manifest /nonexistent/m.tsv >/dev/null 2>&1; test $? -eq 1 && echo REFUSAL-PATH-STILL-LIVE"
---

Three failures from one small instrument fix, all of them cheap to repeat.

## 1. The consistency check you are about to add may fire on a HEALTHY pipeline

The plan was: *refuse when the input table's row count disagrees with the manifest's count.*
Writing it revealed the input had **299 physical lines for a 296-row population** — one record's
last column held a multi-line error message, so it spanned four lines, and a downstream
`if len(cols) < 6: continue` had been swallowing the fragments in silence for as long as the
producing branch existed.

⛔ A `wc -l`-based guard would have read 299 against 296 and **failed a correct pipeline**. That is
the most expensive kind of gate to ship: a false positive trains the operator to bypass it, after
which the guard is worse than absent, because everyone believes it is running.

**Rule:** before comparing two counts, verify the FORMAT of both. A reconciliation is only as good
as the weaker artifact's structure. Here that meant fixing the producer (escape line breaks and
tabs per cell) and making the consumer **REFUSE on a malformed line instead of skipping it** —
the silent skip is what hid the corruption in the first place.

## 2. A refusal that fires is not a refusal that fires *for your reason*

The first proof of the row-count guard used a conveniently-available stale table. It refused — and
the demonstration was worthless, because that table was *also* malformed, so the **malformed-line**
guard fired first and the row-count path never executed.

**Rule:** construct an input that can trigger **exactly one** path. Here: take the healthy table
and delete one row. Well-formed, wrong count, one possible verdict. Do this per guard, and also
prove the ACCEPT path (an explicit `--expect-rows` escape hatch) so you have shown the gate is
usable, not merely strict.

## 3. Read the exit code where the process exits

```bash
python3 tool.py --bad-input 2>&1 | sed 's/^/  /'; echo "exit=$?"   # ← reports sed's status: 0
```

Every refusal in the first evidence run printed `exit=0`. An evidence harness that cannot see a
failure — written while repairing an instrument that could not see a failure.

```bash
out=$(python3 tool.py --bad-input 2>&1); rc=$?     # capture first, then report
```

or use `${PIPESTATUS[0]}`. Note that Python's `raise SystemExit("message")` exits **1** and prints
to stderr, so a bare `2>/dev/null` hides the very message you are verifying.

## 4. Bonus, from the same change: diff size is a design review

The first version of the per-cell escape was `" ".join(value.split())` — idiomatic, and it rewrote
**79 lines** of a 296-row artifact that had one real defect, because it collapsed internal
whitespace runs everywhere. Narrowed to a one-for-one replacement of `[\r\n\t]`, the diff became
exactly the two defective records.

**Rule:** if the diff is much larger than the defect, you have written two changes. Beyond the
review risk, an oversized diff destroys the before/after as evidence — you can no longer point at
it and say *this, and only this, is what I did*.

Related: [[feedback_instrument_needs_ground_truth]],
[[an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green]],
[[a-negative-control-can-disable-the-assertion-it-is-testing]].
