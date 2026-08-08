---
id: a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness
title: A published measurement must carry the identity of what produced it — otherwise "is this number still true?" costs an archaeology and only ever yields a suspicion
answers:
  - "how do I know whether a tracked corpus report still describes my tree"
  - "my measurement report looks stale — how do I prove it before re-running everything"
  - "what should a characterization or benchmark report record besides the number"
  - "the report says it was generated against parseability_probe — is that enough provenance"
  - "how do I tell a stale-and-wrong number from a stale-but-still-correct one"
  - "is a corpus timeout a parser defect or an instrument artifact"
  - "my corpus pass count did not move — does that mean nothing changed"
  - "how many runs do I need to re-measure a corpus honestly"
  - "why did the same corpus report two different pass counts on two different days"
  - "should I track the debug-build or release-build result of a corpus run"
tags: [measurement, provenance, corpus, staleness, instrument-honesty, verification]
date: 2026-08-08
status: current
evidence: stimuli/run_external_corpus.sh (the "Instrument identity" report block); stimuli/sv/characterization/characterization.md; docs/tasks/SV-CORPUS-GRAD.md § "axis-2 freshness audit" + leaf .10; docs/book/src/gate-flow.md §7.10; docs/tasks/artifacts/sv_corpus_grad/axis2_remeasure/
reverify: "test \"$(shasum -a 256 grammars/systemverilog.ebnf | cut -d' ' -f1)\" = \"$(grep -m1 'grammars/systemverilog.ebnf' stimuli/sv/characterization/characterization.md | grep -oE '[0-9a-f]{64}')\" && echo REPORT-DESCRIBES-THIS-GRAMMAR"
---

**A tracked report published `16 336 files, 59.3 % pass` and named its instrument as *"generated
against `parseability_probe`"*.** That basename is not an identity: not which build, not which
grammar, not which commit. So the only available way to challenge the number was to compare the git
commit dates of two unrelated files — the report was committed 2026-07-25, the grammar last changed
2026-07-26, therefore the report provably could not describe `HEAD`.

⛔ **That reconstruction is expensive, easy to skip, and it can only ever produce a SUSPICION.** When
the corpus was actually re-run, the verdict set was **identical**: zero pass→fail, zero fail→pass
across all 16 336 files. The report was *stale by provenance and correct by content*, and **nothing
short of re-measuring could separate those two states.**

⭐ **The rule: publish the identity of the instrument as content hashes, next to the number.** For a
corpus runner that is the sha256 of the parse binary, the grammar and the generated parser, plus the
measuring `HEAD`. Then the staleness question is one command — re-hash three files — instead of date
archaeology, and a reader knows whether to quote the number or re-run it.

⚠️ **The instrument's SETTINGS belong in that identity too, because one column of the result is
about the instrument rather than the subject.** Measured over the same 16 336 files:

| run | binary | per-file budget | pass | fail | timeout |
|---|---|---|---|---|---|
| A | debug | 20 s | 9 693 | 6 632 | 11 |
| B | debug | 60 s | 9 694 | 6 636 | 6 |
| C | release | 60 s | 9 694 | 6 638 | 4 |

The pass/fail sets are **identical in all three**; only `timeout` moves. ⇒ **a timeout is a
statement about the instrument, never about the parser** — and a report that hides which instrument
ran invites a budget effect to be read as a corpus regression. It also retro-explained a previously
unexplained "+1 jitter" row: a file whose parse straddles a 20 s budget and passes deterministically
at 60 s.

⭐ **Rank re-measurement runs by what each one can RULE OUT, rather than arguing about which single
run is "right".** Reproduce the old conditions first (a difference would then be the subject), then
remove one variable per run (budget, then build). Three cheap runs turn "probably an instrument
artifact" into a measured fact.

⛔ **Compare per-FILE, never per-COUNT.** The headline above moved 9 693 → 9 694: a delta of one,
which nets a heal against a regression and licenses nothing
([[a-rising-pass-rate-is-not-evidence-of-correctness]]). The per-file transition census says
something far stronger — *every* passing file still passes and *every* failing file still fails —
and that is the claim which actually licenses continuing a campaign from its existing worklist.

**Give the diff itself a positive and a negative control before quoting it** — an identity self-join
must report zero changes (and its count must reproduce the tracked headline, which incidentally
proves the preserved baseline really is the artifact behind the report), and one planted status flip
must be caught exactly once, at exactly that file
([[feedback_instrument_needs_ground_truth]]).
