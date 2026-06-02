<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_ebnf_consult_annotation_docs.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: always-consult-the-annotation-reference-set-before-editing-any-ebnf-return-annotation
description: "Before adding/changing any `-> ...` return annotation in a *.ebnf grammar, re-read the authoritative annotation sources AND the extraction memory. Repeated failure mode: re-deriving / mis-using `$N` on Quantified groups (binop_chain `rest:$2`) when the correct SV-proven idiom was already known."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

2026-05-16: user escalated twice that I keep NOT recalling annotation knowledge already proven in `systemverilog.ebnf` and re-investigating from scratch (even asking scope questions) instead of applying the established idiom. This wasted effort and risked papering over a real bug.

**Why:** PGEN's return-annotation language has non-obvious extraction/spread semantics (`$N`, `$N::M[*]`, `$N*`, `$N**`). Misusing them silently produces broken AST (`<invalid_sequence_access>`, empty `text`, envelope-wrapped values). The correct patterns are already documented and battle-tested in `systemverilog.ebnf` (~116 slices) and `return_annotation.ebnf` itself.

**How to apply:** Whenever a PGEN task touches a `*.ebnf` file's return annotations (add/edit a `-> ...`, or debug a bad AST shape), BEFORE editing:
1. Read `grammars/return_annotation.ebnf` (the annotation language definition — extraction `$N::M`, spread `$N*`, flat-spread `$N**`, self-uses `[$1,$2::2*]`).
2. Read `docs/RETURN_ANNOTATIONS_REFERENCE.md` and `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`.
3. Recall [[feedback_quantified_group_extraction]] (Category A/B/C for `X (sep X)*`; binop chains are Category B → use single-star `rest:$2*`, never bare `rest:$2`).
4. Grep `grammars/systemverilog.ebnf` for an analogous already-typed rule and copy its proven idiom rather than inventing one.
5. After regen, verify the real dump with `parseability_probe --parse-dump-ast-pretty` — never trust the annotation by inspection alone. For `@predicate`/`@emit_fact` semantic-runtime directives, also verify the **mechanism** (high-verbosity `--trace`: `PGEN_TRACE_VERBOSITY=high`), not just that outputs look right — a mis-resolved arg makes the predicate **error and silently fall back**, so wrong-reason outputs can masquerade as passes (the "false pass" trap).
6. `@predicate`/`@emit_fact` **arg refs resolve by NAME, not position.** A positional `$N` compiles to `RuleReference("N")` (the `$` is dropped) and is looked up as a nonexistent child "N" → unresolved → directive errors. Use the **named sub-rule ref** `$sub_rule_name` — exactly the proven `systemverilog.ebnf` `@predicate{… args:[…, $class_identifier]}` / `@emit_fact{name:$let_identifier}` idiom. (Positional `$N` is still valid in `-> ...` RETURN annotations — that is a separate, working resolution path. Same `.ebnf`, two different ref-resolution rules.) 2026-05-18 RGX-0084: lost a regen+probe cycle to `@predicate args:[…, $2]`; correct form `$backreference_digits`.

A repo settings.json PreToolUse hook on Edit/Write of `*.ebnf` surfaces this list automatically; treat the reminder as binding, not optional. Related: [[feedback_annotation_no_dquote_escape]], [[feedback_annotation_no_mixed_spread]], [[feedback_codegen_outer_branch_remap]].
