# MEMORY-ARCH: Durable Agent Memory Architecture Adoption

## Metadata

- Tree ID: `MEMORY-ARCH`
- Status: `active`
- Roadmap lane: `Cross-cutting infrastructure — durable, harness-agnostic agent memory + enforcement`
- Created: `2026-06-02`
- Last updated: `2026-06-02`
- Owner: repo-local workflow

## Goal

Adopt the portable, harness-agnostic **Durable Agent Memory Architecture**
standard in pgen so agent memory survives session loss, app/machine crash, a
switch of AI model, and a switch of harness (Claude Code → Codex → Cursor → …),
and is **hard to ignore** (mechanical enforcement). Source standard:
`/Users/richarddje/Documents/github/specforge/MEMORY_ARCHITECTURE.md` (408 lines,
project-agnostic, copy-as-is). Adapt only the documented knobs to pgen's
conventions (commit-msg scheme `PGEN-<FAMILY>-<NNNN>` / `<TREE>-Slice-N`,
`docs/tasks/` as the task-tree dir, `COMMIT.md` as the commit-workflow doc).

Director decisions (2026-06-02):
- Sequencing: AFTER the SV-EXH-PROOF `.7.2` reach campaign reached its logged
  checkpoint (done — see SV-EXH-PROOF `.7.2 CAMPAIGN SUMMARY`).
- Enforcement: FULL (E1 bootstrap pointers · E2 self-check · E3 git hooks · E4 CI).
- Resume pointer: REPLACE the stale 1.37 MB tracked root `MEMORY.md` with the
  bounded ≤~50-line pointer (history stays in git); make it canonical.
- Migration: MIGRATE ALL durable `~/.claude` records into tracked
  `docs/decisions/` now (full untracked-memory exposure closure).

## Non-Goals

- Not replacing the existing task-tree system (layer B) — pgen already has it
  (31 trees + `docs/TASK_TREE.md` + `docs/TASK_TREE_README.md`); this composes
  AROUND it (adds layers A/C/D-discipline + enforcement).
- Not replacing `COMMIT.md` / `LIVE_ACHIEVEMENT_STATUS.md` — the standard must
  COMPOSE with them. The commit-msg hook regex must accept pgen's EXISTING commit
  style (verify against real subjects before activating, so normal work is never
  blocked).
- Not deleting historical `MEMORY.md` content — git already preserves it; we stop
  carrying it forward.

## Context / why now

Factual gap assessment (2026-06-02): pgen is STRONG on layer B (task-trees) and D
(git + `CHANGES.md`), but: layer A is split-brain (stale 1.37 MB tracked root
`MEMORY.md` + the live ~53-line one in untracked `~/.claude`); layer C
(`docs/decisions/`) is MISSING — all durable feedback/decisions (incl. this
session's, e.g. the no-guess discipline born from the `.7.2.10` regression) live
ONLY in `~/.claude` (untracked, harness-local, lost on tool/machine switch);
E1–E4 enforcement absent. The exposure: everything called "memory" this campaign
sits in `~/.claude` and would not survive a machine loss or harness switch.

## Acceptance Criteria

- `MEMORY_ARCHITECTURE.md` at repo root; `README.md` doc-map points at it.
- `docs/decisions/` (layer C) with INDEX; ALL durable `~/.claude` records migrated
  to dated ADR records.
- Root `MEMORY.md` is the bounded ≤~50-line resume pointer (overwrite-only).
- Enforcement: `scripts/check_memory_architecture.sh` (E2) + `.githooks/`
  pre-commit + commit-msg (E3, `core.hooksPath`) + CI step (E4) + bootstrap
  pointers `AGENTS.md`/`CLAUDE.md`/`.cursorrules`/`.github/copilot-instructions.md`
  (E1) — commit-msg regex VERIFIED to accept pgen's existing subjects.
- Gates demonstrably bite (a bad subject rejected, an over-cap MEMORY.md fails the
  check) + live-docs sync; tree closed through `COMMIT.md`.

## Task Tree

- ID: `MEMORY-ARCH`
  Status: `active`
  Goal: `Adopt the durable memory architecture standard in pgen (full enforcement), composing with the existing task-tree + COMMIT.md systems.`
  Children: `MEMORY-ARCH.1 .. .5`

- ID: `MEMORY-ARCH.1`
  Status: `done` (`PGEN-MEMORY-ARCH-0002`, 2026-06-02)
  Goal: `Author/copy MEMORY_ARCHITECTURE.md to the repo root (from the specforge standard, verbatim where project-agnostic) + add a README.md doc-map pointer to it. Establishes the system-of-record document before the layers/enforcement land.`
  Acceptance: `MEMORY_ARCHITECTURE.md present at root; README.md references it (+ the task-tree + COMMIT.md conventions) so it is discoverable from the tool-neutral entrypoint; committed via COMMIT.md.`
  Verification: `done — copied /Users/richarddje/Documents/github/specforge/MEMORY_ARCHITECTURE.md to repo root VERBATIM (diff -q = identical; 408 lines; the standard is project-agnostic, copy-as-is per its §0). Added README.md pointers in TWO discoverable places: the "Fast Ramp-Up (Read In This Order)" list (MEMORY_ARCHITECTURE.md as #14, with MEMORY.md reframed as "layer A — bounded resume pointer; read first on resume") + the "Active Markdown Index" (MEMORY_ARCHITECTURE.md = the memory/continuity system of record). README already references docs/TASK_TREE.md (layer B) + COMMIT.md (commit workflow), so the tool-neutral entrypoint now routes to the full system. NO code, no release bump. (Per-harness bootstrap pointer files AGENTS.md/CLAUDE.md/etc. land in .4 E1.)`
  Commit: `done — PGEN-MEMORY-ARCH-0002`

- ID: `MEMORY-ARCH.2`
  Status: `pending`
  Goal: `Create docs/decisions/ (layer C) + INDEX.md, and MIGRATE ALL durable records currently in ~/.claude (the feedback_*/project_*/reference_* memory files: standing disciplines, decisions, env quirks, failed-approach learnings) into dated ADR-style records (Context → Decision → Consequences), linked from the related task-trees and the INDEX. Closes the untracked-memory exposure.`
  Acceptance: `docs/decisions/ with INDEX in sync with the record files; every durable ~/.claude memory has a tracked record; records are plain-text self-describing; committed.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-ARCH.3`
  Status: `pending`
  Goal: `Reconcile layer A: REPLACE the stale 1.37 MB tracked root MEMORY.md with the bounded ≤~50-line resume-pointer template (current_commit / active_work_unit+frontier / next_action / in_flight / blockers). History stays in git. Make the tracked root MEMORY.md canonical; the ~/.claude pointer stops being the system of record.`
  Acceptance: `root MEMORY.md ≤ the line cap, overwrite-only resume-pointer shape, no history; points at MEMORY_ARCHITECTURE.md + docs/tasks/ + docs/decisions/; committed.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-ARCH.4`
  Status: `pending`
  Goal: `Enforcement kit: scripts/check_memory_architecture.sh (E2, single source of truth for the invariants); .githooks/pre-commit (runs the check) + .githooks/commit-msg (work-unit-id subject gate) + git config core.hooksPath .githooks (E3); a CI step running the same check (E4); bootstrap pointer files AGENTS.md / CLAUDE.md / .cursorrules / .github/copilot-instructions.md (E1). ADAPT the commit-msg regex to pgen's scheme and VERIFY it accepts real existing subjects (e.g. "SVEXH-Slice-125 (PGEN-...): ...", "PGEN-SV-EXH-PROOF-0136 ...") BEFORE activating, so normal work is never blocked. core.hooksPath is a local-git-behavior change (director-approved).`
  Acceptance: `the check script + hooks + CI step + bootstrap pointers exist and are wired; the commit-msg regex provably accepts pgen's existing commit subjects; committed.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-ARCH.5`
  Status: `pending`
  Goal: `Verify end-to-end: the gates demonstrably BITE (commit-msg rejects a non-compliant subject + accepts a compliant one; check fails when MEMORY.md exceeds the cap or a bootstrap/decisions piece is missing); the install commit itself passes through the newly-active hooks; live-docs sync (TASK_TREE.md, LIVE_ACHIEVEMENT_STATUS.md, CHANGES.md); close the tree.`
  Acceptance: `proof the four gates bite (recorded); CI green; live-docs synced; tree promoted to Completed in docs/TASK_TREE.md.`
  Verification: `pending`
  Commit: `pending`

## Decisions

- `2026-06-02`: full migration + full enforcement + replace-root-MEMORY.md (director).
- `2026-06-02`: sequenced after SV `.7.2` reach campaign (logged checkpoint reached).

## Notes / watch-outs

- Compose, don't replace: COMMIT.md + LIVE_ACHIEVEMENT_STATUS.md + the task-tree
  doctrine already exist and are binding; the standard adds layers A/C + enforcement
  around them.
- The commit-msg regex MUST accept pgen's existing subject styles before
  activation; test against a sample of real `git log` subjects (verify-before-arm).
- Two MEMORY.md locations exist (repo root vs `~/.claude`); `.3` makes the tracked
  root one canonical and migrates the `~/.claude` durable content in `.2`.
- Per [[feedback_no_codebase_change_without_tool_backed_facts]] + the task-tree
  doctrine: scripts/hooks are code → owned by this tree's leaves; verify each gate
  actually bites rather than assuming.
