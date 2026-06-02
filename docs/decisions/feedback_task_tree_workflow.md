<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_task_tree_workflow.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Task-tree workflow installed in PGEN 2026-05-14
description: PGEN task-tree workflow. BINDING DOCTRINE (2026-05-17, non-negotiable): NO code change without a task-tree leaf owning it first. PNT selects from `docs/TASK_TREE.md` active trees first; only pure non-code single-slice doc work stays on bare `PGEN-<FAMILY>-<NNNN>` IDs.
type: project
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
Installed 2026-05-14. Files:

- `docs/TASK_TREE_README.md` — portable installation guide (copy to other projects).
- `docs/TASK_TREE.md` — PGEN-local workflow + active task tree index + PNT selection rules.
- `docs/tasks/TEMPLATE.md` — copy-this-for-new-tree skeleton.
- `docs/tasks/*.md` — one file per top-level task tree.

**Why:** the SV typing campaign grew to 116+ slices in a flat TaskList with no structure. For the next multi-slice lanes (per-parser-family mdBooks + integration contracts), the task-tree gives container-vs-leaf decomposition, current-frontier discipline, durable blockers, cross-session continuity, and explicit acceptance criteria per leaf.

**⛔ BINDING CODE-CHANGE DOCTRINE (user, 2026-05-17 — non-negotiable, supersedes the old "one-shot code fix may skip the tree"):**
It is **strictly forbidden to make ANY code change unless a task-tree
leaf owns it first.** "Code change" = `grammars/*.ebnf` (grammars ARE
code), Rust (`rust/`), codegen, generated artifacts, shape-contract
manifests (`rust/test_data/ast_shape_contract/*.json`), or anything
altering parser/codegen/generated behavior. A leaf must exist & own the
change BEFORE code is touched; implement only that leaf; full COMMIT.md.
Rationale: task-tree ownership improved code review + code quality
tremendously. Authority: `docs/TASK_TREE.md` "Code-Change Doctrine" +
`COMMIT.md` + live-book `quality-and-closure-model.md`. Even a
one-line grammar tweak needs a leaf. When unsure (mixed change) →
treat as code → require a leaf.

**How to apply:**

- **Any code change (incl. one-shot/one-line)**: a task-tree leaf must own it first — create/extend a tree (or add a leaf to an active one) from `docs/tasks/TEMPLATE.md`, list in `docs/TASK_TREE.md`, work that leaf. Commit subject names the leaf ID alongside the slice ID: `(PGEN-<FAMILY>-<NNNN>, leaf <TREE>.<path>)`.
- **Pure non-code single-slice work ONLY** (live-docs/contracts/books/trackers/workflow-docs — no grammar/Rust/codegen/manifest): the bare `PGEN-<FAMILY>-<NNNN>` slice ID is still fine, no `docs/tasks/` update needed. This is the *only* remaining task-tree-exempt category, and it never includes code.
- **PNT order**: read `docs/TASK_TREE.md` first; pick from the active tree's frontier. Fall back to roadmap-level PNT against `LIVE_ACHIEVEMENT_STATUS.md` if no active tree applies.

**Initial active trees** (created 2026-05-14):
- `VHDL-MDBOOK`, `RTL-FE-MDBOOK`, `RTL-CE-MDBOOK`, `SVPP-MDBOOK` — stand up per-parser mdBooks (SV book serves as the layout template).
- `VHDL-CONTRACT-BODY`, `RTL-FE-CONTRACT-BODY`, `RTL-CE-CONTRACT-BODY`, `SVPP-CONTRACT-BODY` — body out the per-parser integration contracts to SV parity.

**Historical SV slice campaign is NOT retrofitted**: ~116 SV slices, plus regex/rtl_const_expr/rtl_frontend/sv_preprocessor/vhdl typing campaigns, all completed before this workflow installation. Their history lives in `CHANGES.md`, the git log, and the per-grammar `calibration_history` field of the ast_shape_contract manifests.
