---
id: ebnf-frontend-architecture
title: How PGEN parses .ebnf — the hand-written Rust frontend (authoritative) vs the generated cross-check
answers:
  - "how does PGEN parse a .ebnf grammar file"
  - "is the EBNF parser hand-written or generated"
  - "what is src/ebnf_frontend.rs vs generated/ebnf.rs vs grammars/ebnf.ebnf"
  - "how do I extend the EBNF meta-grammar syntax (add a new annotation or construct)"
  - "do I need a bootstrap regen to change the EBNF grammar syntax"
  - "how does a .ebnf file flow into the AST pipeline"
  - "where is the EBNF tokenizer and how are optional [ ] / @ annotations tokenized"
tags: [ebnf, frontend, bootstrap, meta-grammar, architecture, reference]
date: 2026-06-06
status: current
evidence: src/ebnf_frontend.rs (hand-written parser); src/ast_pipeline/mod.rs:1355 transform_from_raw_ast; src/main.rs (load_grammar_bundle / emit_rust_frontend_raw_ast_json call sites); grammars/ebnf.ebnf (documented meta-grammar + seed); rust/Makefile:579-592 (one-time generated/ebnf.rs seed)
reverify: `grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs`
---

**The authoritative EBNF parser is HAND-WRITTEN Rust: `src/ebnf_frontend.rs`.** It — not the
generated parser — is what actually parses `.ebnf` grammars in the pipeline.

## The flow
```
.ebnf  ──ebnf_frontend.rs──▶  raw_ast token envelope  ──transform_from_raw_ast──▶  grammar IR  ──▶ generator/codegen
        (hand-written)         (JSON tokens)            (ast_pipeline/mod.rs:1355)   (ASTNode + Annotations)
```
- Entry: `parse_ebnf_file_to_raw_ast_envelope` / `parse_ebnf_text_to_raw_ast_envelope` (ebnf_frontend.rs:14/25).
  `scan_top_level_rules` collects each rule's before-rule annotations + body; `convert_scanned_rule`
  emits `["rule", name]` and `["semantic_annotation", [name, payload]]`; `tokenize_rule_expression`
  (ebnf_frontend.rs:490) tokenizes the body into `["group_open","("]`, `["group_close",")"]`,
  `["operator","?"]`, `["rule_reference", id]`, `["regex", …]`, `["quoted_string", …]`,
  `["quantifier", …]`, inline annotations (`semantic_annotation_inline` /
  `semantic_annotation_mid_sequence`), and inline `->` return-annotation tokens.
- Consumed by `ast_pipeline/mod.rs::transform_from_raw_ast` (the envelope's `next_step`) → the grammar IR.
- Call sites: `src/main.rs` (`load_grammar_bundle`, `emit_rust_frontend_raw_ast_json` ≈ :989/:1744/:1949)
  and `src/ast_pipeline/stimuli_generator.rs:11000`.

## `generated/ebnf.rs` is only a NON-FATAL cross-check
`generated/ebnf.rs` (the `EbnfParser`) is generated *from* `grammars/ebnf.ebnf` via a one-time seed
(Makefile:579-592: `ebnf.ebnf → ebnf.json → ebnf.rs`). In the pipeline it is used **only to
cross-check** the hand-written output (ebnf_frontend.rs:52-64), and that check is **soft**: it
*warns* on mismatch, is **skipped** when inline semantic annotations are present, and only hard-errors
if `PGEN_EBNF_FRONTEND_REQUIRE_GENERATED_VERIFY` is set (off by default — `generated_verify_required`).

⭐ **Stronger than "soft": NO product path reads its AST at all** (consumer census,
`LANG-CAPABILITY-AUDIT.10.5`). Every consumer takes a **verdict** — `parse_with_ebnf` →
`.is_ok()` (parser_registry.rs:627), `parse_with_ebnf_detail` → `Result<(),String>` (:633), the
frontend cross-check → `if let Err(..)` (ebnf_frontend.rs:70), `ebnf_dual_run_diff.rs:157` → a
diagnostic report. The one AST consumer, `parse_with_ebnf_ast_json` (:648), is called only by
`parse_harness_equivalence.rs` — the interpreter-vs-generated self-consistency oracle. `ebnf` is
also the only tracked generated parser with **neither** a `docs/contracts/` integration contract
**nor** a `rust/test_data/ast_shape_contract/` manifest. ⇒ changing the SHAPE the meta-grammar
produces moves no published surface; what it must preserve is **recognition**. See
[[ebnf-self-hosting-what-it-means]] for what the self-hosting number is and is not, and
[[probe-a-metagrammar-change]] for testing a meta-grammar edit end-to-end without touching the repo.

## Consequence — extending the EBNF meta-grammar syntax
To add a new annotation/construct to the `.ebnf` syntax (e.g. a new bracketed annotation), the real
change is to the **hand-written `ebnf_frontend.rs`** (the tokenizer + the top-level scan) plus
`transform_from_raw_ast` (carry the new token onto the rule/element IR), then the generator. **You do
NOT need a bootstrap regen of `generated/ebnf.rs`** — that's a soft cross-check (gate it on the new
construct like inline-semantic does, or regen it separately as documentation/seed). `grammars/ebnf.ebnf`
is the documented meta-grammar + the seed; keep it in sync, but it is not the live parser.

Note: in `tokenize_rule_expression`, a `[` in a rule body is tokenized as an **optional group**
(`[X]` → `"(" … ")" "?"`). A new `[`-prefixed construct must branch in that `'['` case (peek the next
char) — and is unambiguous against optional because no rule-expression atom begins with that char.
