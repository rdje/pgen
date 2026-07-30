---
id: bootstrap-builtin-annotation-parsers
title: "The two BUILTIN annotation parsers — hand-written chicken-and-egg breakers; `builtin_*.ebnf` are their INFERRED specs, NOT the full annotation grammars, and `parse_bootstrap` NEVER fails"
answers:
  - "what are the builtin parsers in PGEN and why do they exist"
  - "what is bootstrap mode / --bootstrap-mode and what runs in it"
  - "what is grammars/builtin_semantic_annotation.ebnf vs grammars/semantic_annotation.ebnf"
  - "what is grammars/builtin_return_annotation.ebnf vs grammars/return_annotation.ebnf"
  - "which annotation payloads does PGEN actually accept"
  - "how does PGEN parse @transform: str::parse::<usize>().unwrap_or(0)"
  - "can an annotation payload span multiple lines or nest braces"
  - "does the bootstrap annotation parser ever reject a payload"
  - "where is the hand-written semantic annotation parser code"
tags: [bootstrap, builtin, annotations, chicken-and-egg, semantic-annotation, return-annotation, architecture, reference]
date: 2026-07-30
status: current
evidence: "grammars/builtin_semantic_annotation.ebnf (its own header: 'Inferred EBNF/behavioral spec for the hand-written bootstrap semantic parser … Source of truth for this spec is implementation behavior in rust/src/ast_pipeline/unified_semantic_ast.rs … intentionally permissive … not equivalent to grammars/semantic_annotation.ebnf … never hard-fails on syntax'); rust/src/ast_pipeline/unified_semantic_ast.rs:286 parse_bootstrap — three-way TOTAL classification: `trimmed.contains(\"::parse::<\") && trimmed.contains(\">().unwrap_or(\")` -> TransformExpr, else parse_structured_payload -> Structured, else Raw{content: trimmed}, with NO error path; rust/src/ast_pipeline/unified_return_ast.rs is the return-annotation twin; rust/Makefile regex_parser_bootstrap generates generated/ebnf.rs with --bootstrap-mode. MEASURED (LANG-CAPABILITY-AUDIT.10.5, session #228): of the 148 distinct annotation lines across every tracked grammar, grammars/semantic_annotation.ebnf REJECTS 9 — the 4 `@transform:` Rust-code payloads (return_annotation.ebnf x3, regex.ebnf x1), 3 `@generate:` expression payloads, and the 2 multi-line `@dispatch_table:` blocks — while the BUILTIN grammar models all three classes by construction (transform_candidate / structured_payload / raw_payload)."
reverify: "sed -n '286,340p' rust/src/ast_pipeline/unified_semantic_ast.rs; grep -n 'any_text :=\\|raw_payload :=\\|transform_candidate :=' grammars/builtin_semantic_annotation.ebnf; ./rust/target/release/parseability_probe --parse semantic_annotation <(printf '@transform: str::parse::<usize>().unwrap_or(0)')"
---

PGEN ships **two hand-written builtin annotation parsers**. They exist to break a
**chicken-and-egg**: the annotation parsers are themselves generated *from* grammars, and
generating any grammar requires parsing that grammar's annotations. Something must parse
`@entry: true` before a generated annotation parser exists.

| builtin | implementation | inferred spec |
|---|---|---|
| semantic annotations (`@name: payload`) | `rust/src/ast_pipeline/unified_semantic_ast.rs` (`parse_bootstrap`, :286) | `grammars/builtin_semantic_annotation.ebnf` |
| return annotations (`-> …`) | `rust/src/ast_pipeline/unified_return_ast.rs` | `grammars/builtin_return_annotation.ebnf` |

## ⛔ The `builtin_*.ebnf` files are SPECS, not generation sources

They are **inferred behavioural descriptions of hand-written Rust**, written down so the
bootstrap surface is reviewable. Their own header says it: *"Source of truth for this spec
is implementation behavior in `unified_semantic_ast.rs`"*, *"intentionally permissive"*,
*"not equivalent to `grammars/semantic_annotation.ebnf`"*. Editing a `builtin_*.ebnf` does
**not** change bootstrap behaviour — the Rust does.

## ⭐ `parse_bootstrap` is TOTAL — it never fails

Three-way classification of the trimmed payload, in order, with no error path:

1. **`TransformExpr`** — iff the text contains **both** `::parse::<` **and** `>().unwrap_or(`.
   A procedural marker test, *not* parsing. This is how `@transform: str::parse::<usize>().unwrap_or(0)` is accepted.
2. **`Structured`** — else if the bootstrap structured-value parser succeeds (strings,
   numbers, booleans, null-likes, `$refs`, identifiers, **arrays and objects**).
3. **`Raw { content }`** — else. Opaque. Empty payloads land here too.

⇒ **the payload contract is "structured if possible, opaque otherwise, never reject."**
`builtin_semantic_annotation.ebnf` mirrors that with `raw_payload := any_text` where
`any_text := /(.|\n)*/` — **an opaque matcher that spans newlines** — and
`structured_object := "{" … "}"` recursing through `structured_payload` for **balanced
nested braces**.

## The trap this closes

`grammars/semantic_annotation.ebnf` looks like the authority on annotation payloads. It is
not. Measured over the real corpus it **rejects 9 of the 148** annotation lines tracked
grammars actually contain — the whole `@transform:` family included. It is a large
(112-rule) *aspirational* grammar carrying annotation names, lambda expressions, URL
references and semantic versions that PGEN never writes. **When you need to know what an
annotation payload may be, read the builtin grammar and `parse_bootstrap` — not
`semantic_annotation.ebnf`.** See [[ebnf-self-hosting-what-it-means]] and
[[ebnf-frontend-architecture]].
