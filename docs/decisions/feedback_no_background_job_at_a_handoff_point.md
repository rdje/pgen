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

⛔ **Detection is a PROPERTY, not a vocabulary** — corrected the same day it shipped (`.77`(b),
`-0137`). The first cut matched command lines against a list of project tokens and the director
refuted it in one question: `caffeinate -i -t 300`, spawned by the session with its cwd in the repo,
was invisible to it. Measured at pid level on one control job (`sleep 40`, two open repo handles, no
project token in its command line): the vocabulary form reported **nothing**, and in the quiet state
reported a **false positive**. It was wrong in both directions at once.

Now one `lsof` call decides: an **open handle** under the repo (any fd, `txt` included) or a command
line naming this checkout ⇒ project work, **blocking**; **cwd** under the repo with **zero** handles
⇒ **advisory**, non-blocking. The split is principled rather than an allowlist — *cwd is inherited*,
so every helper a session spawns has one, while real work holds a file.

⚠️ **Honest limit.** It censuses this uid, so a project job running as another user is not seen; and
a process that has closed every handle, sits outside the repo and names no repo path is invisible to
both arms. Its caller-exclusion rests on an ancestry walk that can truncate when an intermediate
shell exits, so this script and the harness wrapper are also excluded by name.

Related: [[feedback_answer_your_own_technical_questions]] — the rule is mine to discharge at every
handoff without being asked again.
