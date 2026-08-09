---
id: a-slow-diagnostic-may-be-hung-not-working-hard
title: A slow diagnostic and a hard problem look identical from the outside — check WHERE the time goes before believing what the slowness implies about the subject
answers:
  - "my corpus sweep has been running for minutes — is the corpus slow or is my script broken"
  - "how do I tell a hung instrument from a genuinely expensive measurement"
  - "why did my regex-based tokenizer helper hang on large inputs"
  - "how should I strip trailing whitespace and comments from a byte prefix in a diagnostic"
  - "my script spawns subprocesses but none are running — what does that mean"
  - "is it safe to report 'the corpus is slow' when a sweep times out"
tags: [diagnostics, instrument-honesty, redos, corpus, root-cause, performance]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.15; docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_begin_family.py (`strip_trailing_trivia`, and the comment recording the regex it replaced); docs/tasks/artifacts/sv_corpus_grad/gen_block_family/README.md "Two instrument defects"
reverify: "python3 -c \"import sys; sys.path.insert(0, 'docs/tasks/artifacts/sv_corpus_grad/gen_block_family'); from sweep_begin_family import strip_trailing_trivia as s; assert s(b'begin   // c\\n  ') == b'begin'; assert s(b'x /* a */ /* b */ ') == b'x'; print('LINEAR-SCANNER-OK')\""
---

**A diagnostic that is spinning and a diagnostic that is doing expensive work present the same
symptom: it hasn't finished yet.** The difference matters enormously, because the natural
conclusion drawn from an unfinished sweep is a claim *about the subject* — "the corpus is slow",
"these files are huge", "the parser is pathological on this input" — and that claim gets recorded
as a measured fact.

In `SV-CORPUS-GRAD.3.15` a sweep over 331 corpus rows was killed after **three minutes at 100 %
CPU**. The comparable tracked tool, `cluster_rejects_valid.py`, probes the *same* file set with the
same per-file subprocess in **5 seconds**. That discrepancy was the whole finding — but only
because it was checked rather than explained away.

## The tell

```bash
ps -o pid,etime,time,command -p <pid>     # CPU time vs elapsed time
ps -eo pid,command | grep '[p]arseability_probe'   # are the subprocesses even there?
```

**CPU time ≈ elapsed time with an EMPTY subprocess table.** A sweep whose work is forking probes
and waiting on them should be mostly *idle* in its own process. If the driver is pegged at 100 %
and nothing it claims to spawn is running, the time is being spent inside the driver — it is not
measuring anything at all.

## The cause here, and why the obvious spelling is a trap

The task was "drop the trailing run of whitespace and comments so the caller sees a real token".
The obvious spelling is a regex:

```python
re.sub(rb"(?:\s+|//[^\n]*|/\*.*?\*/)+\Z", b"", consumed)   # ⛔ catastrophic backtracking
```

Two quantified alternatives nested under an anchored `+` gives the engine exponentially many ways
to partition the same trailing run, and `\Z` forces it to try them all before failing. On short
inputs it is instant, so it passes every casual test and only detonates on real data. The fix is
not a cleverer pattern but a **linear backwards scan** — an explicit loop that consumes whitespace,
`/* … */`, and `// …` runs, and terminates on every input by construction.

## The generalization

- **When a diagnostic is slow, locate the time before interpreting the slowness.** A hung
  instrument silently converts into a false fact about the thing it was measuring.
- **Anchored `+` over alternating quantifiers is a hazard wherever it appears**, not only in user
  input handling — a trusted-input helper inside a diagnostic is exactly where nobody looks.
- **Compare against a sibling instrument on the same input** when one exists. Here the 5 s vs 3 min
  gap between two tools doing the same probes was unambiguous, and no reasoning about file sizes
  could have produced it.

Sibling records: [[feedback_instrument_needs_ground_truth]] (an instrument needs controls on its
OUTPUT) and [[a-copied-diagnostic-covers-only-where-it-was-pasted]] (an instrument's documented
coverage and its real coverage are independent). This one covers a third axis: an instrument that
never returns is not a slow measurement, it is an absent one.
