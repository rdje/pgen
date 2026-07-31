# MEMORY-ARCH: Durable Agent Memory Architecture Adoption

## Metadata

- Tree ID: `MEMORY-ARCH`
- Status: `done` (CLOSED 2026-06-02)
- Roadmap lane: `Cross-cutting infrastructure — durable, harness-agnostic agent memory + enforcement`
- Created: `2026-06-02`
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
  Status: `done` (CLOSED 2026-06-02; `PGEN-MEMORY-ARCH-0001..0006`) — all 5 leaves done; layers A/B/C/D + enforcement E1–E4 in place, gates proven to bite, untracked-memory exposure closed.
  Goal: `Adopt the durable memory architecture standard in pgen (full enforcement), composing with the existing task-tree + COMMIT.md systems.`
  Children: `MEMORY-ARCH.1 .. .5`

- ID: `MEMORY-ARCH.1`
  Status: `done` (`PGEN-MEMORY-ARCH-0002`, 2026-06-02)
  Goal: `Author/copy MEMORY_ARCHITECTURE.md to the repo root (from the specforge standard, verbatim where project-agnostic) + add a README.md doc-map pointer to it. Establishes the system-of-record document before the layers/enforcement land.`
  Acceptance: `MEMORY_ARCHITECTURE.md present at root; README.md references it (+ the task-tree + COMMIT.md conventions) so it is discoverable from the tool-neutral entrypoint; committed via COMMIT.md.`
  Verification: `done — copied /Users/richarddje/Documents/github/specforge/MEMORY_ARCHITECTURE.md to repo root VERBATIM (diff -q = identical; 408 lines; the standard is project-agnostic, copy-as-is per its §0). Added README.md pointers in TWO discoverable places: the "Fast Ramp-Up (Read In This Order)" list (MEMORY_ARCHITECTURE.md as #14, with MEMORY.md reframed as "layer A — bounded resume pointer; read first on resume") + the "Active Markdown Index" (MEMORY_ARCHITECTURE.md = the memory/continuity system of record). README already references docs/TASK_TREE.md (layer B) + COMMIT.md (commit workflow), so the tool-neutral entrypoint now routes to the full system. NO code, no release bump. (Per-harness bootstrap pointer files AGENTS.md/CLAUDE.md/etc. land in .4 E1.)`
  Commit: `done — PGEN-MEMORY-ARCH-0002`

- ID: `MEMORY-ARCH.2`
  Status: `done` (`PGEN-MEMORY-ARCH-0003`, 2026-06-02)
  Goal: `Create docs/decisions/ (layer C) + INDEX.md, and MIGRATE ALL durable records currently in ~/.claude (the feedback_*/project_*/reference_* memory files: standing disciplines, decisions, env quirks, failed-approach learnings) into ADR-style records, linked from the INDEX. Closes the untracked-memory exposure.`
  Acceptance: `docs/decisions/ with INDEX in sync with the record files; every durable ~/.claude memory has a tracked record; records are plain-text self-describing; committed.`
  Verification: `done — created docs/decisions/ + migrated ALL 54 durable ~/.claude memory records (45 feedback_ + 7 project_ + 2 reference_; ~236 KB; MEMORY.md itself excluded — that is layer A, handled by .3) into tracked files, each preserving the original content VERBATIM (self-verified: tail-after-banner diff vs source = 0 mismatches across all 54) with a provenance banner noting the migration + that ~/.claude is now a cache. Generated INDEX.md DERIVED from each record's frontmatter (name/description → 54 rows, category from filename prefix) so it cannot drift. The records keep their original frontmatter (name/description/metadata.type) → plain-text + self-describing per the standard. This CLOSES THE UNTRACKED-MEMORY EXPOSURE: every durable discipline/decision (incl. this campaign's no-guess discipline, parser-agnostic mandate, director-role, fix-hierarchy, etc.) is now git-tracked + harness-portable. NO code, no release bump. Note: the records are migrated as-is (not re-authored into strict Context→Decision→Consequences prose) — faithful preservation was prioritized over reformatting; future edits can ADR-ify individual records. Frontier → MEMORY-ARCH.3 (root MEMORY.md → bounded pointer).`
  Commit: `done — PGEN-MEMORY-ARCH-0003`

- ID: `MEMORY-ARCH.3`
  Status: `done` (`PGEN-MEMORY-ARCH-0004`, 2026-06-02)
  Goal: `Reconcile layer A: REPLACE the stale 1.37 MB tracked root MEMORY.md with the bounded ≤~50-line resume-pointer template. History stays in git. Make the tracked root MEMORY.md canonical.`
  Acceptance: `root MEMORY.md ≤ the line cap, overwrite-only resume-pointer shape, no history; points at MEMORY_ARCHITECTURE.md + docs/tasks/ + docs/decisions/; committed.`
  Verification: `done — overwrote the root MEMORY.md (was 18562 lines / 1.37 MB of stale carried-forward session prose, last real update 2026-05-05 SV-Slice-19 era — pure layer-A/B/D content conflated, the standard's #1 anti-pattern) with the bounded resume-pointer template: now 23 lines / 2.1 KB (≤ the ~50-line target + the 60-line check cap). Contains How-to-resume (points at MEMORY_ARCHITECTURE.md + README + docs/tasks/ + docs/TASK_TREE.md + COMMIT.md + docs/decisions/INDEX.md + LIVE_ACHIEVEMENT_STATUS.md + CHANGES.md) + the OVERWRITE-only Current-state block (latest_commit / active_work_unit+frontier / next_action / in_flight / blockers) + a short Other-open-threads note (SV-EXH-PROOF paused at logged checkpoint). The old 1.37 MB content is PRESERVED in git (HEAD~:MEMORY.md = 1369466 bytes; last touched commit b632c680) — not deleted, just no longer carried forward, per the standard §6. The tracked root MEMORY.md is now canonical layer A; the ~/.claude/.../memory/MEMORY.md (53-line harness index) is now a cache (its durable referenced records were migrated to docs/decisions/ in .2). NO code, no release bump. Frontier → MEMORY-ARCH.4 (enforcement kit).`
  Commit: `done — PGEN-MEMORY-ARCH-0004`

- ID: `MEMORY-ARCH.4`
  Status: `done` (`PGEN-MEMORY-ARCH-0005`, 2026-06-02)
  Goal: `Enforcement kit E1–E4 (check script + .githooks + core.hooksPath + CI + bootstrap pointers), commit-msg regex adapted to pgen + verified against real subjects before arming.`
  Acceptance: `the check script + hooks + CI step + bootstrap pointers exist and are wired; the commit-msg regex provably accepts pgen's existing commit subjects; committed.`
  Verification: `done — E2: scripts/check_memory_architecture.sh (single source of truth) checks MEMORY_ARCHITECTURE.md present, MEMORY.md ≤ cap (default 60, env MEMORY_POINTER_LINE_CAP), AGENTS.md+CLAUDE.md point at the standard, docs/TASK_TREE.md + docs/tasks/ exist, docs/decisions/ + INDEX present + index-not-empty-while-records-exist; exits nonzero on any breach. E1: 6 bootstrap pointer files (AGENTS.md, CLAUDE.md, GEMINI.md, .cursorrules, .windsurfrules, .github/copilot-instructions.md) — each a 14-line pointer to README + MEMORY_ARCHITECTURE.md + the layers; all 6 reference MEMORY_ARCHITECTURE.md. (CLAUDE.md confirmed NOT previously tracked → safe additive pointer, no behavior conflict.) E3: .githooks/pre-commit (runs the check) + .githooks/commit-msg + `git config core.hooksPath .githooks` (armed; director-approved local-git-behavior change). E4: .github/workflows/memory-architecture-gate.yml runs the SAME check script + a commit-subject validation (un-bypassable backstop; --no-verify doesn't reach CI). COMMIT-MSG REGEX — VERIFY-BEFORE-ARM (per [[feedback_no_codebase_change_without_tool_backed_facts]]): tested candidates against ALL 1699 historical subjects. A STRICT work-unit-id pattern would have BLOCKED 1280/1699 (~75% — pgen's history is mostly free-form prose; the id-in-subject convention is recent). So chose a PERMISSIVE leading-token regex `^[A-Za-z][A-Za-z0-9._-]+` (accepts all 1699 real subjects, rejects only blank/whitespace/punct-led); the work-unit-id convention is carried by discipline + COMMIT.md + the task-tree doctrine + review + CI, NOT a brittle hook (matches the standard §9 E3 "honest limit"; documented in both the hook + the check script). GATES PROVEN TO BITE: commit-msg REJECTS "   bad subject" + ACCEPTS "PGEN-MEMORY-ARCH-0005 ..."; the check FAILS at MEMORY_POINTER_LINE_CAP=5 (MEMORY.md=23 lines) + passes at default; self-check green with all layers present. NO Rust/grammar/generated code (shell scripts + yaml + md only); no release bump. Frontier → MEMORY-ARCH.5 (end-to-end verify incl. this tree's own commits passing the armed hooks + live-docs sync + close).`
  Commit: `done — PGEN-MEMORY-ARCH-0005`

- ID: `MEMORY-ARCH.5`
  Status: `done` (`PGEN-MEMORY-ARCH-0006`, 2026-06-02)
  Goal: `Verify end-to-end: the gates demonstrably BITE; the install commits pass through the armed hooks; live-docs sync; close the tree.`
  Acceptance: `proof the gates bite (recorded); live-docs synced; tree promoted to Completed in docs/TASK_TREE.md.`
  Verification: `done — E2E battery (all from the live repo): [1] scripts/check_memory_architecture.sh green at HEAD ("memory-arch: OK"); [2] core.hooksPath=.githooks (armed); [3] the .4 install commit 0ed9d83c was created WITH the hooks active (pre-commit printed "memory-arch: OK") — live proof; [4] commit-msg REJECTS "   no id here" + ACCEPTS "PGEN-MEMORY-ARCH-0006 ..."; [5] check FAILS at MEMORY_POINTER_LINE_CAP=5 (MEMORY.md=23 lines) + passes at default; [6] layer inventory: A=MEMORY.md 23 lines, B=33 task-trees + TASK_TREE.md, C=54 decision records + INDEX, D=git 1700 commits + CHANGES.md, E1=6/6 bootstrap pointers. LIVE-DOCS SYNCED: LIVE_ACHIEVEMENT_STATUS.md (refreshed stale header + 2 new tracker notes: MEMORY-ARCH closed + the SV .7.2 reach consolidation), CHANGES.md (per-leaf), docs/TASK_TREE.md (tree promoted active→Completed). NOTE: E4 CI workflow added + is valid YAML running the same check, but a live CI run is not exercised from this local session (the local hooks + the in-session check ARE exercised; CI will run server-side on push per push-pacing). Tree CLOSED. NO Rust/grammar/generated code; no release bump.`
  Commit: `done — PGEN-MEMORY-ARCH-0006`

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
