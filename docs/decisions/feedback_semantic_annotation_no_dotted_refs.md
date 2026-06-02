<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_semantic_annotation_no_dotted_refs.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: semantic-annotation-rule-reference-dotted-form-supported-since-3-3-4-a-1
description: "HISTORICAL (pre-`.3.3.4.a.1` 2026-05-20): the semantic-annotation language accepted ONLY simple `$name` / `$1` rule references in directive payloads — no `$name.body`. The fix was non-obvious because the actual runtime parser is the HAND-ROLLED `StructuredSemanticValueParser::parse_rule_reference` in `rust/src/ast_pipeline/unified_semantic_ast.rs`, NOT the EBNF-generated `semantic_annotation_parser.rs` (which is the EBNF-language surface, not the grammar-directive-payload path). The runtime resolver always supported dotted walks (`resolve_named_semantic_reference` / `parse_semantic_reference_segments`). NOW (`.3.3.4.a.1`, `PGEN-SV-EXH-PROOF-0026`): the hand-rolled `parse_rule_reference` accepts `.<ident>` chains to unbounded depth, and `semantic_annotation.ebnf::rule_reference_name` regex is extended in lockstep so the language definition mirrors the runtime."
metadata: 
  node_type: memory
  type: reference
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Original trap (pre-fix, 2026-05-20 MVP-0 `.3.3.4.a`):** `@export_to_library: {kind:package, name_from:$name.body}` produced "Directive '@export_to_library' expects a structured object payload" — opaque, looked like the whole annotation parsed wrong. Original root cause analysis pointed at `grammars/semantic_annotation.ebnf:411`'s `rule_reference_name := /([a-zA-Z_][a-zA-Z0-9_]*|[0-9]+)/` (no `.`). The MVP-0 worked around it by surfacing the nested scalar at the shape's top level via the return-annotation property-access (`-> { …, body: $4.body }`), letting the directive use simple `$body`.

**Refined root-cause (during `.3.3.4.a.1`, 2026-05-20):** the EBNF `rule_reference_name` regex was a RED HERRING — the bootstrap-parser-surface for directive payloads is NOT the regenerated `semantic_annotation_parser.rs`. It is the HAND-ROLLED `StructuredSemanticValueParser::parse_rule_reference` in `rust/src/ast_pipeline/unified_semantic_ast.rs`, invoked via `UnifiedSemanticAST::parse_bootstrap` → `parse_structured_payload`. Extending only the EBNF regex was no-op for grammar directive payloads. The fix had to land in the hand-rolled Rust source.

**Current state (post-`.3.3.4.a.1`, release 1.0.123):** `$name.body` and arbitrary-depth dotted chains (`$a.b.c.d.e…`) work in any semantic-annotation directive payload. The hand-rolled `parse_rule_reference` accepts `.<ident>` chains to unbounded depth (`while peek_char() == Some('.')` with no max-iteration cap). EBNF surface is extended in lockstep so the language definition mirrors the runtime. A regression test `bootstrap_semantic_dotted_rule_reference_depth_is_structurally_unbounded` exercises 64 segments and locks the no-depth-limit guarantee. NEXT (`.3.3.4.a.2`, queued): the same surfaces extended with `[<digits>]` indexed-access (`$items[0].name`, `$matrix[0][1]`, mixed) — same unbounded-depth guarantee, own leaf.

**How to apply:**
1. When adding a semantic-annotation directive that needs a nested value: USE `$field.subfield…` DIRECTLY. No more shape workaround needed.
2. The grammar directive payload uses `unified_semantic_ast.rs::parse_rule_reference` (hand-rolled), NOT `semantic_annotation_parser.rs` (the EBNF-language surface, regenerated from `semantic_annotation.ebnf`). When changing what `$<ref>` accepts, BOTH surfaces must be extended in lockstep — the runtime path AND the language definition — so they stay consistent. This is the architectural clarification `.3.3.4.a.1` surfaced; future "rule_reference surface" changes need to touch both.
3. `build.rs` does NOT track `generated/semantic_annotation_parser.rs` as a `rerun-if-changed`, so editing the bootstrap parser doesn't auto-trigger a release-binary rebuild. After regenerating it, `touch rust/src/lib.rs` (or use `cargo clean -p pgen`) to invalidate cargo's incremental cache before testing.
4. The return-annotation language has ALWAYS supported `$N.field` property access (`property_access_expression` in `grammars/return_annotation.ebnf`). That's a separate parser, used in `->` output mappings. Don't confuse the two surfaces.

**Related memories:** [[feedback_annotation_no_dquote_escape]] (similar bootstrap-parser quirk — silent fail on `\"` escape), [[feedback_annotation_no_mixed_spread]] (mixed-spread arrays), [[feedback_ebnf_consult_annotation_docs]] (the binding read-the-docs discipline — but ALSO check the hand-rolled `unified_semantic_ast.rs::StructuredSemanticValueParser` before assuming EBNF tells the whole story; that's the catch this slice taught).
