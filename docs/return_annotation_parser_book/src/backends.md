# Backends: Bootstrap vs Generated

The return-annotation parser is special among PGEN parsers: it has **two backends**, exposed as
`ParserBackend::Bootstrap` and `ParserBackend::Generated`. Understanding why explains how the family is
built and which backend you get.

## The chicken-and-egg problem

PGEN generates a parser *from* a `.ebnf` — but generating that parser code requires reading the
grammar's **return annotations** to know how each rule lowers its match. So to generate the parser
*for the return-annotation language itself*, PGEN would already need a return-annotation parser. That
is circular.

PGEN breaks the cycle with a **bootstrap path**:

- `grammars/builtin_return_annotation.ebnf` is a **bootstrap-safe** grammar contract. It is an
  *implementation-accurate* (deliberately not idealized) description of a small, hand-written parser —
  it documents the accepted syntax and the known parser constraints, with its source of truth being the
  Rust implementation in `rust/src/ast_pipeline/unified_return_ast.rs`.
- Because the bootstrap parser is hand-written, it does **not** depend on a generated
  return-annotation parser, so PGEN can use it to generate everything else.

This is why the README states that the annotation parsers are *"generated with bootstrap mode only"*.

## The two backends

### Bootstrap backend (`ParserBackend::Bootstrap`)

- The hand-written parser in `rust/src/ast_pipeline/unified_return_ast.rs`.
- Documented by `grammars/builtin_return_annotation.ebnf`.
- **Always part of the published contract** — it has no build-feature prerequisite.
- It applies the input normalization described in [Grammar and Scope](grammar-and-scope.md) (trim,
  strip a leading `->`, empty → passthrough).

### Generated backend (`ParserBackend::Generated`)

- The generated parser at `generated/return_annotation_parser.rs`, produced from the full language
  grammar `grammars/return_annotation.ebnf` via the bootstrap generator (see
  [Build Recipe](build-recipe.md)).
- Part of the published contract **when `generated_parsers` support is present** in the build.
- Implements the full `2.0.0` language surface this book describes.

## Which backend should I use?

You normally should **not** choose a backend by linking a parser module directly. Use the stable
embedding API (`parse_annotation` / `parse_annotation_result` / `parse_annotation_named`) with
`AnnotationFamily::Return`, and let the host route to a backend; pass an explicit `ParserBackend` only
when you specifically need to pin one (for example, to compare bootstrap vs. generated behavior in a
test). The two backends are kept in agreement by the annotation contract gates — both are expected to
accept the same language and produce the same shapes for the published surface.

See [Public API](public-api.md) for the entry points and selectors, and
[Schema and Versioning](schema-and-versioning.md) for the backend contract and notable shape changes.
