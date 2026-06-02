<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_commit_workflow_strict.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_commit_workflow_strict
description: User-set 2026-05-25 (emphatic, repeated, non-negotiable) — the commit workflow described in COMMIT.md is STRICT, non-negotiable, must be run after every FULL completion of a task/slice/lane and BEFORE continuing or switching to the next one. Purpose is recoverability from session/computer/application crashes — overlooking it puts the project's continuity at risk.
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set discipline (2026-05-25, verbatim):**

> "Ok here is the commit workflow again
>
> It is described in COMMIT.md and shall be strictly followed.
> The appropriate live-docs and/or live-book ought to be amended.
> The commit workflow is not for fun, it is there to be able to recover swiftly, seamless, flawlessly following a session lost, a session crash, a computer crash or an application crash.
> It must be run following the full completion of a task, slice, lane, and before continuing or switching to the next one.
> So it is critical for the wellbeing of the project, so you should not, ever overlook it.
> It shall be strictly followed, no compromise, non-negotiable."

## What this binds

**Source of truth:** `COMMIT.md` at the repo root. Re-read it before every commit. Do not rely on memory of its contents — it can change.

**Trigger:** every full completion of a task, slice, or lane. Even pure-docs slices. Even one-line edits. Even diagnosis-only slices that produced no code change.

**Forbidden:** continuing or switching to the next task/slice/lane without first running the workflow on the current one. Batching multiple slices into one commit (or none) defeats the recoverability purpose.

**Purpose (non-obvious):** the workflow exists specifically so that ANY disruption — Claude session lost / context blown, terminal crash, OS crash, power failure, application crash — leaves the repo in a recoverable, navigable, accurately-documented state. `git log` + the tracked live-docs are the post-crash continuity surface. If they're out of sync with the working tree, recovery is broken.

## The workflow (canonical, per COMMIT.md as of 2026-05-14)

1. **Ensure task is complete and tested.**
2. **Clippy flow** when Rust/generated Rust amended (`make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`). Skip cleanly if no Rust touched.
3. **Update tracked docs as needed** — systematic, not optional:
   - `CHANGES.md` (changelog)
   - `DEVELOPMENT_NOTES.md` (technical notes)
   - `MEMORY.md` (in-repo memory — distinct from auto-memory)
   - `LIVE_ACHIEVEMENT_STATUS.md` (Done / Mostly Done / In Progress / Not Started snapshot — must be reviewed every commit; if a row changes, summarise + state effect in completion message; if no row changes, still display + say "tracker unchanged")
   - `docs/reference/RUST_CODEBASE_ANALYSIS.md` when Rust architecture changes
   - `README.md` when objective / canonical flow / paths / commands / doc map changes
   - `docs/book/` when a user-facing or developer-facing surface already represented changes (or warrants a new chapter/section); run `make -C rust SHELL=/bin/bash mdbook_docs_gate` when the book changes
   - `docs/contracts/*` when stable integration surface / build requirements / validation gate / bug-report bundle changes
   - Convert any repo-internal absolute checkout path to a relative path before commit.
4. **Write concise commit message** to `git_message_brief.txt` (must stay untracked).
5. **Stage only intended tracked files** with `git add <files>`.
6. **Commit** with `git commit -F git_message_brief.txt`.
7. **Clear message file** with `: > git_message_brief.txt`.
8. **Verify post-conditions:**
   - `git ls-files --error-unmatch git_message_brief.txt` must fail (untracked)
   - `wc -c git_message_brief.txt` must be 0
   - `git status --short` shows only expected state
9. **Completion message reports:**
   - the commit ID
   - the exact commit message
   - the list of tracked files included
   - the current live-status snapshot
   - whether the snapshot changed or stayed unchanged

## Task-tree-leaf naming requirement

When the slice belongs to a task-tree leaf (under `docs/tasks/`), the commit subject or first body line MUST name the leaf ID alongside the PGEN slice ID, e.g.:

```
SVEXH-Slice-NN (PGEN-SV-EXH-PROOF-NNNN, leaf SV-EXH-PROOF.X.Y.Z): <subject>
```

Also: update the owning `docs/tasks/<TREE>.md` (status, verification log, commit log, frontier, decisions). Update `docs/TASK_TREE.md` Active Task Trees table only if the current frontier changes.

## Anti-patterns I (Claude) must not slip into

- "It's a small docs edit, I'll batch it with the next change" → NO. Commit per slice, every time.
- "The workflow is overkill for this slice" → NO. The recoverability invariant doesn't accept exceptions.
- "I'll skip step 8 because git status looks clean to me" → NO. Verify every post-condition explicitly.
- "I'll skip the live-status review because I don't think it changed" → NO. The review IS the verification that it didn't change; display the snapshot either way.
- Reading the chat for what COMMIT.md probably says → NO. Re-read COMMIT.md from disk before every commit. It can change.

## Cross-references

- `COMMIT.md` (repo root) — source of truth, re-read every commit
- [[feedback_task_tree_workflow]] — task-tree ownership doctrine; complementary to commit workflow
- [[feedback_push_pacing]] — push is its own decision (currently overridden — never push without explicit per-push authorization)
- [[feedback_regex_book_live]] — book lockstep is part of step 3 when user-facing surfaces change
- [[feedback_user_is_director_not_engineer]] — commit decisions (when/whether) are MINE within agreed principles; this workflow is one of the agreed principles
