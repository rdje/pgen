---
name: feedback_no_background_job_at_a_handoff_point
description: STANDING RULE — at any handoff point (whenever the director is ready to /exit), no background job may still be running; census it and kill what is left before declaring the workspace handoff-ready
metadata:
  node_type: memory
  type: feedback
---

**Director standing rule, 2026-08-30, verbatim:** *"From now on as a rule please ensure that at a
handoff point, that is whenever I am ready to /exit, that there is no background job running."*

## What it binds

A handoff point is any moment the workspace is declared ready to leave — above all the
`✅ DONE (clean — safe to /clear)` banner of the pause-signal directive. At that moment:

- **No project-owned background job may still be running.** Not "it will finish on its own", not
  "it only writes to `tmp/`". Census it, kill it, and verify the census is empty.
- **Kill the children too.** Measured while adopting this rule: killing a ladder's parent left
  **two `parse_alloc_census_probe` children running** — the parent's death does not propagate, and
  a census taken only of the parent reports clean while the work continues.
- This EXTENDS the existing handoff bar rather than replacing it. Clean tree + everything committed
  was already required; a running job is now equally disqualifying.

## Why — it is not hygiene, it is a measured data-loss class

`PGEN-ENGINE-UNIVERSAL-SERVICES-0134` opened on a tree layer A recorded as clean and which was
being **rewritten underneath it**: a `chain.sh` from a session that no longer existed had run for
**1 h 24 m** and was overwriting **three TRACKED artifacts of an already-CLOSED leaf**
(`.67`(c)) at a different measurement configuration, its log gone with the session. It was caught
only because two `wc -c` samples seconds apart disagreed.

⇒ **A background job outlives the session that spawned it, and its output paths are tracked files.**
The next session inherits a dirty tree it cannot explain, with no log, and the honest reconstruction
costs more than the job was worth. Committing to a durable layer does not protect against this:
the corruption happens *after* the commit, to files the commit already blessed.

## How it is discharged — executable, not remembered

```bash
bash scripts/check_no_background_jobs.sh     # 0 = clean · 1 = candidates listed
```

⛔ **Deliberately NOT a doctrine and NOT wired into `.githooks/pre-commit`.** A commit may
legitimately land while a job runs — the pause-signal directive's own rule 3 says to commit
completed work FIRST and then wait. A commit-blocking form would punish exactly the behaviour that
directive asks for. This is a handoff-time operator check and nothing else.

The census claims a process only if its command line names this checkout **or** its own working
directory is inside it, because the first cut reported two unrelated `linkedspec` processes — which
is precisely how an operator learns to ignore a census. Both arms are exercised: it has been
observed going RED on a real project job and GREEN once that job is killed.

⚠️ **Honest limit.** It matches on command line plus cwd, so a job whose cwd is elsewhere *and*
whose command line never spells the repo path is missed, and a process whose cwd cannot be read is
treated as not ours. Its caller-exclusion rests on an ancestry walk that **can truncate** when an
intermediate shell has already exited — observed once, not reliably reproducible — so this script
and the harness wrapper are additionally excluded by name. Neither exclusion can hide a real
project job, which is the property that matters.

Related: [[feedback_answer_your_own_technical_questions]] — the rule is mine to discharge at every
handoff without being asked again.
