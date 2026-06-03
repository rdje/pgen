---
id: ast-two-surface-construction
title: PGEN builds typed AST objects on TWO surfaces that must change in lockstep
answers:
  - "where are typed AST objects constructed in PGEN"
  - "why is the _meta carrier (A5) a coordinated multi-surface change"
  - "what is the two-surface architecture (runtime interpreter vs codegen)"
  - "what must I edit to add a sibling key to every AST object"
  - "which files build the AST object map"
tags: [architecture, codegen, ast, meta-carrier]
date: 2026-06-03
status: current
evidence: rust/src/ast_pipeline/unified_return_ast.rs (object build ~:636-:706, serde_json::Map); rust/src/ast_pipeline/return_annotation_handler.rs (emits object-construction Rust into generated parsers, ~:355); docs/tasks/PARSE-SOTA-A5-meta-carrier-design.md
reverify: grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs
---

PGEN constructs typed AST objects in **two** places, and any change to the object shape
(e.g. adding an additive `_meta` sibling key per A5) must land in **BOTH** in lockstep, or
the bootstrap path and the generated parsers disagree:

1. **Runtime interpreter** — `rust/src/ast_pipeline/unified_return_ast.rs` (`let mut map =
   serde_json::Map::new(); … Ok(Value::Object(map))`, ~:636-:706). Used by the bootstrap
   path. A change here is a shared-lib change, no regen needed.
2. **Codegen** — `rust/src/ast_pipeline/return_annotation_handler.rs` emits the
   object-construction Rust into the generated parsers (~:355 shows `ParseNode { … span:
   0..0 }`). A change here requires **regenerating all 10 `generated/*_parser.rs`**.

This two-surface coupling is the core reason the `_meta` carrier (PARSE-SOTA A5) is a
coordinated, multi-slice effort — not a single edit — and why its rollout is phased
(opt-in default-OFF → real spans → default-ON + migrate all 8 shape contracts). Spans are
also frequently the placeholder `0..0` today, so `_meta.span`/`line_col`/`source_text` need
real span population to be meaningful. See [[feedback_meta_carrier_design]].
