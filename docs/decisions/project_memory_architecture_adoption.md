<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/project_memory_architecture_adoption.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: project-memory-architecture-adoption
description: PLANNED next tree after SV-EXH-PROOF .7.2 — adopt the portable durable-agent-memory standard in pgen (full enforcement). Director-approved sequencing + scope 2026-06-02.
metadata: 
  node_type: memory
  type: project
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**Director decision 2026-06-02:** adopt the Durable Agent Memory Architecture standard in pgen, as its OWN task tree (proposed id `MEMORY-ARCH`), **AFTER** the in-flight SV-EXH-PROOF `.7.2` reach work reaches its decision point (`.7.2.18`). Sequencing = "finish .7.2 first". Enforcement = "yes, full enforcement" (hooks E3 + CI E4, adapted to pgen's commit scheme).

**Source standard:** `/Users/richarddje/Documents/github/specforge/MEMORY_ARCHITECTURE.md` (408 lines, project-agnostic, copy-as-is). 4 durability properties; 4 layers (A resume-pointer `MEMORY.md` ≤~50 lines overwrite-only · B task-trees · C `docs/decisions/` ADRs · D git/CHANGELOG); write-path (route every durable thing to a layer + commit before turn ends); read-path (bounded resume); §9 enforcement E1–E4; §9.1 reproduce-anywhere kit; §11 adoption checklist. specforge implemented it as leaves `.1`–`.5` (closing commit 6cec44db).

**pgen gap assessment (facts, 2026-06-02):**
- B task-trees — ✅ STRONG (31 files under docs/tasks/ + docs/TASK_TREE.md + docs/TASK_TREE_README.md). pgen already exceeds the standard's assumption here.
- D git + CHANGES.md — ✅ present + disciplined (COMMIT.md).
- A resume pointer — ⚠️ SPLIT-BRAIN: the live memory I use is `~/.claude/projects/-Users-richarddje-Documents-github-pgen/memory/MEMORY.md` (~53 lines, UNTRACKED harness home dir = the #1 anti-pattern); ALSO a stale 1.37 MB `MEMORY.md` at the pgen repo root (last touched May 5, unmaintained).
- C decision records — ❌ MISSING. All durable feedback/decisions (incl. this session's, e.g. feedback_no_codebase_change_without_tool_backed_facts) live ONLY in ~/.claude → invisible to other harnesses, lost on tool switch, not in git. THIS IS THE KEY EXPOSURE the adoption closes.
- E1 bootstrap pointers — ❌ no AGENTS.md / CLAUDE.md / .cursorrules / .github/copilot-instructions.md.
- E2/E3/E4 — ❌ no scripts/check_memory_architecture.sh, no .githooks, no CI gate, core.hooksPath unset.

**Planned MEMORY-ARCH tree (mirror specforge .1–.5, adapted to pgen):**
- `.1` add `MEMORY_ARCHITECTURE.md` at repo root (copy verbatim) + README doc-map pointer.
- `.2` create `docs/decisions/` (layer C) + INDEX.md; MIGRATE the durable facts out of `~/.claude` memory into dated ADR records (the high-value, exposure-closing step). The `~/.claude` `feedback_*`/`project_*`/`reference_*` files are the migration source set.
- `.3` reconcile layer A: replace the stale 1.37 MB root `MEMORY.md` with the bounded ≤~50-line resume-pointer template (history stays in git). Decide how the harness-home MEMORY.md relates (likely the tracked one becomes canonical).
- `.4` enforcement kit: scripts/check_memory_architecture.sh (E2) + .githooks/{pre-commit,commit-msg} + `git config core.hooksPath .githooks` (E3) + CI step (E4) + the harness bootstrap pointer files (E1). **Adapt the commit-msg regex to pgen's scheme** (`PGEN-<FAMILY>-<NNNN>` / leaf-id like `SVEXH-Slice-N`) — VERIFY the existing commit style already passes before activating, so it doesn't block normal work. core.hooksPath is a local-git-behavior change (flagged + approved).
- `.5` verify gates bite (commit-msg rejects a bad subject / accepts a good one; check fails when MEMORY.md over cap) + live-docs sync + close.

**Watch-outs:** (a) pgen ALREADY has its own COMMIT.md workflow + LIVE_ACHIEVEMENT_STATUS.md + the task-tree doctrine — the standard must COMPOSE with these, not replace them. (b) The root `MEMORY.md` is 1.37 MB tracked — demoting it is a large tracked-file change; preserve history (git already has it). (c) Reconcile two MEMORY.md locations (repo root vs ~/.claude). (d) Per [[feedback_user_is_director_not_engineer]] this is a real structural change → task-tree-owned first; per [[feedback_no_codebase_change_without_tool_backed_facts]] verify the hook regex against real commit subjects before activating.
