# Welcome — PGEN Return-Annotation Parser Integration Reference

This is the per-parser integration reference book for PGEN's **`return_annotation`** parser, the
parser for the small language that shapes what a generated parser *returns*. It is the canonical
AST-integration surface for anyone consuming the return-annotation parser's output, and it sits
alongside the platform mastery book (`docs/book/`) and the other per-parser books (regex,
SystemVerilog, SystemVerilog preprocessor, VHDL, rtl_frontend, rtl_const_expr, json), all linked from
the platform book's [Parser Families](../../book/src/parser-families.md) chapter.

## What a return annotation is

In a PGEN grammar, a rule may carry a **return annotation** — the `-> …` payload that says how that
rule's match should be lowered into the returned AST. For example, in `grammars/json.ebnf`:

```ebnf
object_property := property_key ':' expression
-> {key: $1, value: $3}
```

The text after `->` (`{key: $1, value: $3}`) is **return-annotation source**. The `return_annotation`
parser is the parser for *that* mini-language. It turns the annotation text into a structured
**annotation AST** that the PGEN code generator consumes to emit the rule's lowering code. So:

- the **grammar** describes the *tree* the parser builds;
- the **return annotation** describes how to *reshape and label* that tree into the value the rule
  returns;
- the **`return_annotation` parser** is what reads the annotation language itself.

This is the first of PGEN's annotation grammars; its sibling is the `semantic_annotation` parser (the
`@…` steering language). Both are core platform grammars, not side utilities.

## Why this matters to a consumer

You interact with the return-annotation parser in two ways:

1. **As a grammar author** — you write `-> …` payloads. This book's
   [Grammar and Scope](grammar-and-scope.md), [References and Literals](references-and-literals.md),
   [Objects and Arrays](objects-and-arrays.md), and
   [Operators](operators.md) chapters are the precise, example-rich reference for what you may write.
2. **As a tooling integrator** — you call the embedding API to parse annotation strings and walk the
   resulting annotation AST. The [AST Envelope](ast-envelope.md), [Backends](backends.md), and
   [Public API](public-api.md) chapters are for you.

## Status and proof

`return_annotation` is a **`Done`** family in the live tracker. Its claim is backed by the
return-annotation support gate (`return_annotation_support_gate`, which now includes the
auto-derived `return_annotation_exhaustiveness_gate`: grammar-driven coverage closure, stimuli-module
parity, and a generated-parse-tree → typed-AST audit), the annotation contract gate, and the
return-runtime-semantics gate. See [Schema and Versioning](schema-and-versioning.md) for the gates and
the notable shape changes.

## What this book covers

- [Build Recipe](build-recipe.md) — how to generate, build, and exercise the parser.
- [Grammar and Scope](grammar-and-scope.md) — exactly what the language accepts (and what it does not).
- [References and Literals](references-and-literals.md) — `$N`, `$text`/`$0`, strings, numbers,
  booleans, `null`, identifiers.
- [Objects and Arrays](objects-and-arrays.md) — object/array literals and the cons-list shape.
- [Operators](operators.md) — extraction (`::`), spread (`*`), flatten-spread (`**`), property
  access (`.`), index access (`[…]`).
- [AST Envelope Structure](ast-envelope.md) — the parsed annotation node shapes.
- [Backends: Bootstrap vs Generated](backends.md) — the two parser backends and when each is used.
- [Public API](public-api.md) — the stable embedding entry points, selectors, and diagnostics.
- [Schema and Versioning](schema-and-versioning.md) — grammar version, gates, and notable shape changes.
- [Glossary](glossary.md).

## Authoritative sources behind this book

- `grammars/return_annotation.ebnf` — the language (currently version `2.0.0`).
- `grammars/builtin_return_annotation.ebnf` — the bootstrap-safe variant (implementation-accurate to
  `rust/src/ast_pipeline/unified_return_ast.rs`).
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` — the downstream contract.
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and `docs/RETURN_ANNOTATIONS_REFERENCE.md` — the
  deep normative detail this book curates and cross-links.
