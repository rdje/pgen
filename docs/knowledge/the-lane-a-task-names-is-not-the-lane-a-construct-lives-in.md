---
id: the-lane-a-task-names-is-not-the-lane-a-construct-lives-in
title: A task is cut from one lane, but a defect belongs to a CONSTRUCT — parameterize the sweep by lane, or ship a fix that reports complete while missing rows nothing will ever surface
answers:
  - "I fixed every row in my worklist — how do I know the fix is complete"
  - "should my corpus diagnostic hard-code the profile and manifest it was written for"
  - "does a grammar ruling for sv_2017 also settle the verilog_2005 lane"
  - "why would rows in a second profile lane never appear in my cluster map"
  - "how do I stop a per-lane fix from looking finished when it is partial"
tags: [corpus, adjudication, diagnostics, scope, profiles, instrument-honesty]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.15; docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_result.txt (19 rows, sv_2017) vs sweep_result_v2005.txt (6 rows, verilog_2005, disjoint); stimuli/sv/adjudicate_external_corpus.py (`EXTRA_PINNED` and `V2005_LRM_PINNED`)
reverify: "comm -12 <(grep -oE '[a-z0-9_/.-]+\\.(sv|v)$' docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_result.txt | sort -u) <(grep -oE '[a-z0-9_/.-]+\\.(sv|v)$' docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_result_v2005.txt | sort -u) | wc -l | grep -qx '0' && echo LANES-DISJOINT"
---

**A task is scoped to a worklist; a defect is scoped to a construct.** Those two boundaries are
unrelated, and when a project keeps more than one adjudication lane — two LRM editions, two
profiles, two families — the worklist a leaf was cut from silently defines the fix's coverage.

In `SV-CORPUS-GRAD.3.15` the leaf was cut from the `sv_2017` axis-2 worklist, and the resume
pointer named that lane too. The sweep that found the family was written **parameterized**
(`--manifest`, `--profile`) rather than hard-coded to the lane in hand. Pointed at the
`verilog_2005` manifest, it returned **6 further rows — all iverilog, and disjoint from the 19**.
The other edition's grammar gives the identical verdict, so the *edition* was never the variable;
the **scope of the question** was.

## Why nothing would have caught it

- A **pass-rate** nets movement across a lane and says nothing about a lane it did not run.
- A **cluster map** and a **family table** are each built from one manifest, so the second lane's
  rows are not "unranked" — they are absent.
- The fix's own before→after measurement would have looked clean and complete, because it would
  have measured the lane it fixed.

This is failure in the **passing direction**: the missing rows produce no red anywhere.

## The rule

**Parameterize the diagnostic by lane instead of forking it per lane.** The cost here was ten
lines of `argparse`; the alternative — a second copy — only ever covers the lane it was pasted into
([[a-copied-diagnostic-covers-only-where-it-was-pasted]]).

Then, before calling a construct-level fix complete, run it against **every** lane that adjudicates
the same language family and state the result for each — including the lanes that came back empty,
since "measured zero" and "never asked" are indistinguishable in a report that omits them.

⚠️ Resolve a relative `--manifest` against the repository root, not the caller's cwd, so one
command line means one thing from any directory.

Sibling record: [[feedback_sv_strict_lrm_compliance_default]] — the per-edition ruling still has to
be taken per edition; parameterizing the *instrument* is what makes asking cheap, not what makes
the answer automatic.
