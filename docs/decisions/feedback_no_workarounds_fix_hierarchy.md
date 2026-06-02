<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_no_workarounds_fix_hierarchy.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_no_workarounds_fix_hierarchy
description: "STANDING DISCIPLINE — never use workarounds/bandaids when fixing; follow a strict fix hierarchy starting with semantic annotations, ending with parser-agnostic engine extensions; never compromise / no-signoff solutions"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Hard prohibition (user, 2026-05-23, emphatic):** never use workarounds, quick-and-dirty fixes, or bandaids. They are anti-signoff. Avoid like the plague.

**The fix hierarchy — in strict order, try each before the next:**

1. **Use existing semantic annotations** (`@emit_fact`, `@predicate`, `@open_scope`/`@close_scope`, `@export_to_library`/`@import_from_library`, `@fact_kind:`, `@predicate_def:`, branch-policy, phase predicates).
2. **Use the existing semantic store** (multi-index fact store, scope tree, `resolve_path`, transactional rollback, composed predicates) — the universal store is rich; query and gate.
3. **Add a new semantic-annotation feature** — extend the annotation language itself with a new general primitive (e.g. a new predicate primitive, a new directive, a new ref form) — parser-agnostic, available to every grammar.
4. **Add a new semantic-store feature** — extend the store/runtime with a new general capability (e.g. fan-out emission, parent-context reference, list/exists predicate, structured-value resolution) — parser-agnostic, available to every grammar.
5. **Parser-agnostic engine enhancement** — only when none of (1)–(4) can be made to apply even with extensions; must benefit every parser, never grammar-specific.

**Never:** SV-specific engine code; bandaid grammar tweaks that hide a deeper problem; "good enough for now" patches; symptomatic fixes that don't address the root cause. Each fix must be sign-off-quality at the appropriate layer.

**How to apply** — when faced with a defect:
- First, FULLY understand the root cause ([[feedback_root_cause_before_fix_code_last_resort]]).
- Verify the EBNF rule captures the PDF/spec intent ([[reference_sv_defect_taxonomy]] §A2 illustrates the discipline).
- Walk the hierarchy 1→5; document at each step why the lower-numbered approach was insufficient *before* moving up.
- If the fix lands at level 3, 4, or 5, the new primitive must be a GENERAL parser-agnostic feature ([[feedback_ast_pipeline_parser_agnostic]], refined version of [[feedback_prefer_grammar_leave_engine_alone]]).
- The slice's commit message must record which level the fix landed at and why nothing lower-numbered worked.

**Strengthens / refines:** [[feedback_prefer_grammar_leave_engine_alone]] (engine = last resort, parser-agnostic-only); [[feedback_ast_pipeline_parser_agnostic]] (every shared pipeline change must be a general primitive); [[feedback_root_cause_before_fix_code_last_resort]] (understand fully before fixing). This memory now governs the fix-approach step that follows root-cause understanding.

**Cross-referenced from:** `docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md` (How-to-use § amend pending).
