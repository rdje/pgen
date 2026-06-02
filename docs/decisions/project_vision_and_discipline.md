<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/project_vision_and_discipline.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: project_vision_and_discipline
description: User-set 2026-05-25 — the project's vision restated in one paragraph. Parsers are .ebnf files (with return annotations + semantic annotations) compiled by a parser-AGNOSTIC Rust AST pipeline. We already have the tools we need; the discipline is to USE what we have well, not drift to new mechanisms because the existing ones feel insufficient.
metadata:
  type: project
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set vision (2026-05-25, verbatim):**

> "The only thing I want is for parsers to be described as EBNF files with support for Return annotations and semantic annotations and have a Rust AST pipeline generating fast parsers from these EBNF files. The AST pipeline is the thing that is common to those parsers and is and should be, shall remain parser-agnostic. We have add support for several feature to help us debug. We also adding more and more semantic annotation constructs and added a semantic annotation store. To me we have all the tools to move forwards. We might or will need more tools, add more semantic annotation statements or constructs. But we need to keep getting better at using what we already have and not drift away from them."

**The system, decomposed:**

| Layer                        | Surface                                                    | Stays parser-agnostic?      |
| ---------------------------- | ---------------------------------------------------------- | --------------------------- |
| Parser definition            | `grammars/*.ebnf` with return annotations + semantic annotations | n/a (parser-SPECIFIC by design) |
| Generator                    | `rust/src/codegen.rs` + `rust/src/codegen/*.rs`            | **YES — parser-agnostic**   |
| Runtime / AST pipeline       | `rust/src/ast_pipeline/*.rs` (incl. semantic store, runtime, library) | **YES — parser-agnostic**   |
| Debug tooling                | `parseability_probe` (--trace-rules, --dump-rule-call-counts, furthest-position, dump_facts, --explain), HIGH/DBG self-explaining trace levels | **YES — parser-agnostic**   |
| Output                       | Fast parsers, one per .ebnf, sharing the pipeline          | n/a                         |

**Standing inventory (already-built, must be REACHED FOR FIRST):**

*Semantic annotation constructs* — `@emit_fact`, `@predicate`, `@export_to_library`, `@import_from_library`, `@fact_kind:`, `@predicate_def`, `@transform`, `phase:branch`, `view: shaped` / `view: raw`, dotted `$x.y[N]` rule references.

*Semantic store primitives* — multi-indexed fact store; scope tree with parent pointers + archived closed scopes; `has_fact` / `lacks_fact_attribute_equals` / `lacks_fact` predicates; `resolve_path` dotted lookup; transactional rollback; per-rule transaction wrapper; library I/O.

*Debug tooling* — see table above in [[feedback_tools_first_no_guessing]] (the question→tool mapping). Self-explaining trace at HIGH (verdict) and DBG (mechanism) levels.

**The discipline (the binding part):**

1. **Use what's already there before proposing more.** The default move on any new defect is to compose existing semantic annotations + existing debug tools. Only when concrete tool evidence shows EVERY existing combination is insufficient does a new construct or a new tool become legitimate.

2. **New things, when added, are GENERAL.** Per [[feedback_ast_pipeline_parser_agnostic]] + [[feedback_no_workarounds_fix_hierarchy]]. A new annotation construct must benefit every grammar; a new pipeline primitive must be parser-agnostic; never SV-specific (or VHDL-specific, regex-specific, etc.) carve-outs.

3. **The AST pipeline parser-agnostic invariant is sacred.** No grammar-specific knowledge in `rust/src/ast_pipeline/*.rs` or `rust/src/codegen*.rs`. If a fix wants to add such knowledge, the fix is wrong — the right fix is in the grammar or a new general primitive.

4. **Don't drift to "we need a new mechanism" because the existing ones feel insufficient.** They probably aren't insufficient; I just haven't composed them right. Pause + compose + use the tools to verify the composition, before proposing extension.

5. **When extension IS legitimate**, the order is: new semantic annotation construct (level 3) → new semantic store primitive (level 4) → new pipeline primitive (level 5). Each level requires written evidence the lower one was insufficient.

**Cross-references:**
- [[feedback_ast_pipeline_parser_agnostic]] — the parser-agnostic invariant in detail (explicitly includes `stimuli_generator.rs`)
- [[feedback_universal_semantic_store]] — the semantic store is parser-agnostic + multi-indexed + scope-aware
- [[feedback_no_workarounds_fix_hierarchy]] — the L1→L5 ordering
- [[feedback_tools_first_no_guessing]] — the discipline of REACHING FOR tools, not just having them
- [[feedback_prefer_grammar_leave_engine_alone]] — engine = last resort
- [[feedback_user_is_director_not_engineer]] — my role: translate this vision into technical decisions within these principles
