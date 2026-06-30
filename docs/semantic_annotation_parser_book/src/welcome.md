# Welcome — PGEN Semantic-Annotation Parser Integration Reference

This is the per-parser integration reference book for PGEN's **`semantic_annotation`** parser — the
parser for the `@…` language that *steers* how a generated parser behaves. It is the canonical
integration surface for anyone consuming the semantic-annotation parser's output, and it sits alongside
the platform mastery book (`docs/book/`) and the other per-parser books (regex, SystemVerilog,
SystemVerilog preprocessor, VHDL, rtl_frontend, rtl_const_expr, json, and its sibling
[return_annotation](../../return_annotation_parser_book/src/welcome.md)), all linked from the platform
book's [Parser Families](../../book/src/parser-families.md) chapter.

## What a semantic annotation is

A PGEN grammar rule can carry **semantic annotations** — the `@name: value` directives that make the
generated parser *context-aware*. Where a [return annotation](../../return_annotation_parser_book/src/welcome.md)
(`-> …`) shapes the *value a rule returns*, a semantic annotation (`@ …`) shapes the parser's
*behavior and state*: gating a rule on facts already seen, emitting facts into a semantic store,
opening scopes, restricting a rule to a profile, transforming a matched value, or steering stimuli
generation. For example, in a SystemVerilog-style grammar:

```ebnf
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

```ebnf
@emit_fact: type_name
type_declaration := 'typedef' data_type identifier ';'
```

The text after `@` is **semantic-annotation source**. The `semantic_annotation` parser is the parser
for *that* language. Together with `return_annotation`, it is one of PGEN's two core annotation
grammars (not a side utility): the return language describes the *tree*, the semantic language
describes the *parser's memory and decisions*.

## Two things this book documents

The semantic-annotation surface has two layers, and this book covers both:

1. **The value language** — every annotation is `@name: value`, and *value* is a rich literal /
   structured / expression / reference language (`grammars/semantic_annotation.ebnf`, version `2.0`).
   This is what the `semantic_annotation` parser literally parses; see
   [Grammar and Scope](grammar-and-scope.md) and
   [Annotation Values and References](values-and-references.md).
2. **The steering directives** — the catalog of `@`-directives PGEN's AST pipeline *interprets*
   (`@predicate`, `@emit_fact`, `@fact_kind`, `@open_scope`, `@export_to_library` /
   `@import_from_library`, `@profiles`, `@predicate_def`, `@transform`, `@generate`, `@sample` /
   `@probe_sample`, …) and what each one *does*. Their syntax is value-language; their *meaning* is
   the steering contract, governed by the normative spec and the steering control matrix. See
   [Steering Directives](steering-directives.md) and [The Semantic Store](semantic-store.md).

## Status and proof

`semantic_annotation` does not carry a separate top-level live-status row; its maturity is tracked
through the annotation proof spine (`annotation_contract_gate`, `semantic_usage_gate`,
`semantic_runtime_contract_gate`, `semantic_full_contract_gate`) and the normative docs. See
[Schema and Versioning](schema-and-versioning.md).

## What this book covers

- [Build Recipe](build-recipe.md) — generate, build, and exercise the parser.
- [Grammar and Scope](grammar-and-scope.md) — the `@name: value` surface the parser accepts.
- [Annotation Values and References](values-and-references.md) — the value language: literals,
  structured values, expressions, and the `$ref` / `%symbol` / path / URL / type references.
- [Steering Directives](steering-directives.md) — the catalog of directives PGEN interprets.
- [The Semantic Store](semantic-store.md) — the fact/scope lifecycle the steering directives drive.
- [AST Envelope Structure](ast-envelope.md) — the parsed annotation node shapes.
- [Backends: Bootstrap vs Generated](backends.md) — the two parser backends and the two runtime surfaces.
- [Public API](public-api.md) — the stable embedding entry points, selectors, and diagnostics.
- [Schema and Versioning](schema-and-versioning.md) — grammar version, gates, and notable shape changes.
- [Glossary](glossary.md).

## Authoritative sources behind this book

- `grammars/semantic_annotation.ebnf` — the `@name: value` value language (version `2.0`).
- `grammars/builtin_semantic_annotation.ebnf` — the bootstrap-safe variant.
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` — the downstream contract.
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and
  `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` — the deep normative steering semantics this
  book curates and cross-links.
