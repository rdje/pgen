# COMMIT.md

Last updated: 2026-02-28

## Purpose
Define the exact commit workflow for this project so a new AI instance can apply it consistently without re-reading chat history.

## When To Run
Run this workflow after each completed task/activity.

## Files Involved
- `git_message_brief.txt` (must remain untracked)
  - Short, concise commit message file.
  - Used with `git commit -F git_message_brief.txt`.
  - Must be cleared to 0 bytes after commit.
- `CHANGES.md` (tracked)
  - Changelog-style summary of completed work and validation.
- `DEVELOPMENT_NOTES.md` (tracked)
  - Detailed technical notes: root cause, implementation, validation.
- `MEMORY.md` (tracked)
  - Live continuity file for resume/handoff.
- `questions_keep_untracked.txt` (must remain untracked)
  - User backlog/questions for future UG work.
- `generated/` artifacts
  - Regenerable outputs; not authoritative source-of-truth.

## Required Commit Workflow (Exact Order)
1. Ensure task is complete and tested.
2. Run clippy flow when Rust/generated Rust files are amended:
   - `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`
   - strict source lint must pass.
   - generated-parser lint runs too; set `PGEN_CLIPPY_GENERATED_STRICT=1` to fail on generated clippy debt.
3. Update tracked docs as needed (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, others touched by task).
4. Write concise commit message to `git_message_brief.txt`.
5. Stage only intended tracked files (`git add <files>`).
6. Commit with:
   - `git commit -F git_message_brief.txt`
7. Clear message file:
   - `: > git_message_brief.txt`
8. Confirm post-conditions:
   - `git ls-files --error-unmatch git_message_brief.txt` must fail (untracked).
   - `wc -c git_message_brief.txt` must be `0`.
   - `git status --short` must show expected state only.

## Pre-Commit Safety Rules
- Do not add `git_message_brief.txt` to git.
- Do not track `questions_keep_untracked.txt`.
- Do not commit generated artifacts unless explicitly requested.
- Do not use destructive git commands unless explicitly requested.

## Command Template
```bash
# 1) write concise commit message
cat > git_message_brief.txt <<'EOF'
<concise title>

- <brief bullet 1>
- <brief bullet 2>
EOF

# 2) run clippy flow when Rust/generated-Rust changed
make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change

# 3) stage intended files only
git add <tracked-file-1> <tracked-file-2> ...

# 4) commit
git commit -F git_message_brief.txt

# 5) clear message file
: > git_message_brief.txt

# 6) verify
wc -c git_message_brief.txt
git ls-files --error-unmatch git_message_brief.txt >/dev/null 2>&1; echo $?
git status --short
```
