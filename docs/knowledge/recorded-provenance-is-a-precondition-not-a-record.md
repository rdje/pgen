---
id: recorded-provenance-is-a-precondition-not-a-record
title: Recorded provenance is a PRECONDITION, not a record — an artifact that states how it was measured must bind the run that replaces it, or the block is a comment
answers:
  - "why did re-running the corpus change the numbers when the parser did not change"
  - "is it enough to record how a measurement was produced"
  - "how do I stop a re-run from silently re-baselining a tracked oracle"
  - "why does run_external_corpus.sh refuse with exit 5"
  - "what does PGEN_CORPUS_REBASELINE / PGEN_CORPUS_OUT_DIR do"
  - "why is a default argument value dangerous in a measurement script"
  - "how can a burn-down number improve because the machine was busy"
tags: [provenance, gates, measurement-integrity, corpus, external-corpus, doctrine, shell]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.27 (the fix + five controls) and leaf .3.25 (the incident — 6 rows moved to divergence:explained_timeout, timeouts 4 -> 10, on a parser change that provably could not touch them); stimuli/run_external_corpus.sh (reconcile()/DRIFT_LINES, the exit-5 refusal); docs/tasks/artifacts/sv_corpus_grad/timeout_reconfirm_control/run_control.sh (positive+negative ground truth for the serial re-confirmation); docs/book/src/gate-flow.md §5 "The measurement-parameter rule" + §7.11
reverify: "stimuli/run_external_corpus.sh sv 20 8 0; test $? -eq 5 && echo 'REFUSAL LIVE'"
---

A gate that publishes a number into the repository is publishing an **oracle**: downstream work
recomputes from it. PGEN learned to make such artifacts *self-describing* — the external-corpus
characterizations carry sha256s of the parse binary, the grammar and the generated parser, plus the
invocation they were produced with
([[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]]).

**That fix was half a fix.** The provenance block was written by every run and read by none.

## What that costs, measured

`stimuli/run_external_corpus.sh` documented its own bare invocation:

```bash
stimuli/run_external_corpus.sh sv
```

All three tracked artifacts (`sv`, `vhdl`, `sv2005`) record `60 8 0` against
`rust/target/release/parseability_probe`. The script's defaults were **20 s** against
`rust/target/debug/parseability_probe`, and one corpus file parses in **12 s release / 127 s
debug**. So the documented command re-measured the whole corpus under a ~10× slower binary at a 3×
tighter deadline and published over the tracked oracle without a word. `SV-CORPUS-GRAD.3.25` caught
it only because the diff looked wrong: **6 rows moved into `divergence:explained_timeout`, timeouts
`4 → 10`**, on a parser change that provably could not touch them.

⛔ **The direction of the error is the whole point.** A `timeout` row is adjudicated
`divergence:explained_timeout` and therefore *leaves* `unexplained_rejects_valid`. So a slower
machine, a debug binary or a tighter deadline **improves the burn-down** — see
[[a-rising-pass-rate-is-not-evidence-of-correctness]]. Nothing guarded that direction, and the
artifact that could have detected it was sitting right there, correct and unread.

## The root cause is one line of shell, and it generalizes

```bash
TIMEOUT_S="${2:-20}"        # ⛔ destroys the information a drift check would need
```

`${2:-20}` collapses *"the caller asked for 20"* and *"nobody said, so the default is 20"* into a
single value. No downstream check can exist, because the distinction it would test was erased on
line 26. The fix is to capture first and default afterwards:

```bash
ARG_TIMEOUT="${2-}"         # what the CALLER said (possibly nothing)
...                         # reconcile against the artifact, THEN default
```

**Generalizable habit:** in any script that overwrites a tracked artifact, `${N:-default}` at the
top of the file is a smell. Defaulting is a *decision*; make it after you have read what the
previous run decided.

## The shape of the fix

| the caller… | the runner… |
|---|---|
| omits an argument | **adopts** the artifact's recorded value — the correct run becomes the easy run |
| supplies a matching value | proceeds |
| supplies a differing value | **refuses, exit 5**, printing both sets, before parsing anything |
| has no tracked artifact yet | uses script defaults, and says so |

Escape hatches must be explicit and loud: `PGEN_CORPUS_REBASELINE=1` (deliberate new baseline) and
`PGEN_CORPUS_OUT_DIR=<dir>` (measure elsewhere, diff, then promote). This is
`DOCTRINE_ENFORCEMENT.md` §3's **structural** archetype — the invariant is re-derived from the
artifact on every run, so it cannot rot.

⭐ Hard-refusing was affordable only because it was checked: `grep -rn run_external_corpus
--include=Makefile --include=*.mk --include=*.sh --include=*.yml .` returns **no invoker**. A
refusal wired into something an aggregate calls is a different decision.

## Two properties the comparison silently depended on

Closing the parameter half is not enough if the artifact still cannot be compared:

- **Determinism of layout.** `xargs -P` appends in *completion* order, so `results.tsv` row order
  was nondeterministic — `sort -c` fails on the pre-fix file at line 3. A re-run with identical
  verdicts still produced thousands of moved lines, which is a complete explanation of why nobody
  diffed it. Sort the artifact, under `LC_ALL=C`.
- **A verdict that is about the subject.** A `timeout` under `-P 8` is partly a claim about the
  machine, so every timeout is now re-run **alone** before it is recorded.

⚠️ **And extending an artifact's schema is a consumer question, not a style question.** Adding a
duration column to `results.tsv` was the obvious move and would have broken all four readers:
three do `suite, observed, path = line.split("\t")` (a hard 3-way unpack → `ValueError`) and
`cluster_rejects_valid.py` does `if len(cols) != 3: continue` — which would have **silently dropped
every row and reported an empty worklist**. One loud break and one silent one, the silent one being
the generator of the burn-down worklist. Durations went to a sidecar. **Measure the consumers before
widening a format**; see
[[normalize-a-shared-convention-once-at-ingestion-and-enumerate-its-consumers]].

## The rule

> **If an artifact states the conditions it was measured under, the run that would replace it must
> be held to them.** Adopt when the caller is silent; refuse when the caller disagrees. Provenance
> nobody reads back is a comment.
