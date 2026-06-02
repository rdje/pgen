<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_hook_scope.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-hook-scope
description: "PGEN .claude/settings.json hook scope, set by the user 2026-05-16 — PostCompact re-reads ONLY live-docs/live-books/TASK_TREE; PreToolUse re-reads the target *.ebnf + important docs; hooks must NOT hard-code rotting engineering rules."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

User directives for the project `.claude/settings.json` hooks (2026-05-16):

- **PostCompact** `additionalContext` shall instruct re-reading (final
  scope, after the user's 2026-05-16 refinements): (1) `README.md`,
  `SESSION_BOOTSTRAP.md`, `COMMIT.md`; (2) live-docs —
  `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  the auto-memory `MEMORY.md` index; (3) live-books (MDBOOKs) —
  `docs/book/` (top-level) + the per-parser book for the in-flight
  lane; (4) `docs/TASK_TREE.md` + the owning `docs/tasks/<TREE>.md`.
  (The user first said "only live-docs/live-books/TASK_TREE", then
  added README/SESSION_BOOTSTRAP/COMMIT back in — this combined set is
  the agreed final.)
- **PreToolUse** (on `*.ebnf` Edit/Write) shall instruct re-reading the
  **target `*.ebnf` grammar itself** plus the important annotation docs
  (`grammars/return_annotation.ebnf`,
  `docs/RETURN_ANNOTATIONS_REFERENCE.md`,
  `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`) and the memories
  [[feedback_quantified_group_extraction]] +
  [[feedback_ebnf_consult_annotation_docs]].

**Why:** the hooks had hard-coded a specific engineering rule
("binop op-chains MUST use `rest:$2*`, NEVER bare `$2`") that was later
empirically falsified (the real fix is a NAMED op-rule + bare `$2`; see
[[feedback_quantified_group_extraction]]). A hard-coded rule in a hook
**rots silently** and then re-injects the wrong guidance on every edit /
compaction — exactly the flaw the user flagged. The user had it removed
("shall not have been there in the first place").

**How to apply:** hooks point to the authoritative live surfaces and
memories — they must NOT duplicate a specific, falsifiable engineering
rule inline. Keep `parseability_probe --parse-dump-ast-pretty`
verification advice (generic, non-rotting) and the "copy the proven
systemverilog.ebnf idiom" pointer. When changing hook scope, edit
`.claude/settings.json` only on explicit user authorization (the
auto-mode classifier treats it as self-modification), validate the JSON
(`python3 -c "import json;json.load(open('.claude/settings.json'))"`),
and commit as a `PGEN-WORKFLOW-<NNNN>` slice.
