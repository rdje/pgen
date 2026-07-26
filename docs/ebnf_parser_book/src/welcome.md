# Welcome — PGEN EBNF Grammar-Author's Reference

This is the per-grammar reference book for PGEN's **`ebnf`** meta-grammar — the language you use to
*write a PGEN grammar*. Where the other per-parser books (regex, SystemVerilog, VHDL, `rtl_frontend`,
`rtl_const_expr`, json, [return_annotation](../../return_annotation_parser_book/src/welcome.md), and
[semantic_annotation](../../semantic_annotation_parser_book/src/welcome.md)) document the *output* of a
generated parser, this book documents the *input*: the EBNF surface a grammar author types into a
`grammars/foolang.ebnf` file. It is linked from the platform book's
[Parser Families](../../book/src/parser-families.md) chapter.

The authoritative source for the language is the self-hosting meta-grammar `grammars/ebnf.ebnf`
(version `2.0`) together with the Rust EBNF frontend (`rust/src/ebnf_frontend.rs`) and the parser/stimuli
code generators (`rust/src/ast_based_generator.rs`, `rust/src/ast_return_transform.rs`) that actually
*consume* it. **This book documents the surface the codegen really supports — not the aspirational
constructs the meta-grammar self-describes but the generators ignore.** Where a construct is parsed but
not yet acted on, this book says so out loud (see [Rules and Expressions](rules-and-expressions.md) and
the [Glossary](glossary.md)).

## Who this book is for

You are writing or extending a `.ebnf` grammar that PGEN will compile into a parser (and a stimuli
generator). You want to know:

- what terminals and rule constructs are available, and exactly how they behave;
- how a rule's **returned AST** is shaped — both the *implicit/passthrough* default and the explicit
  `-> …` return-annotation language;
- how to *steer* parser-generation behaviour with `@…` semantic annotations;
- how to split a grammar across files with `@include`;
- what the codegen does under the hood, so a surprising parse or shape is debuggable.

## The canonical flow

A PGEN grammar travels a fixed path from text to a working parser:

```text
grammars/foolang.ebnf
        │  EBNF frontend (rust/src/ebnf_frontend.rs)
        ▼
generated/foolang.json          (the raw AST / grammar IR)
        │  code generators (rust/src/ast_based_generator.rs, …)
        ▼
generated/foolang_parser.rs     (the generated recursive-descent PEG parser)
        │
        ├─▶ in-memory stimuli generator  (--generate-stimuli)
        └─▶ generated/foolang_stimuli.rs (optional, --generate-stimuli-module)
```

Everything you write in the `.ebnf` is interpreted along this path. The [Build Recipe](build-recipe.md)
shows the exact commands; the [Codegen Mental Model](codegen-model.md) explains what each construct
becomes.

## What this book covers

- [Build Recipe](build-recipe.md) — generate, parse, lint, and prove a grammar; build this book.
- [Grammar File Structure](grammar-file-structure.md) — the file: rules, operators, comments, whitespace.
- [Terminals](terminals.md) — string/char literals, regex literals, the `builtin_any_char` built-ins, escapes.
- [Rules and Expressions](rules-and-expressions.md) — ordered choice, sequences, grouping, optional,
  and the constructs the codegen does **not** implement.
- [Quantifiers](quantifiers.md) — `?` `*` `+` `{n}` `{n,m}` `{n,}` `{,m}` and the Layer-0 unified engine.
- [Lookaheads](lookaheads.md) — positive `&` / negative `!`, and the `[> … ]` / `[>! … ]` lexical
  follow-restriction.
- [The Implicit / Passthrough Return Policy](return-policy.md) — what a rule returns with **no** `-> …`.
- [Return Annotations](return-annotations.md) — the explicit `-> …` AST-shaping language (overview +
  cross-links to the return_annotation book).
- [Semantic Annotations](semantic-annotations.md) — the `@…` steering language (overview + cross-links to
  the semantic_annotation book).
- [The Include System](includes.md) — `@include` / `include_file` / `include_dir` modular composition.
- [The Bootstrap Path](bootstrap.md) — how the annotation grammars break their own chicken-and-egg.
- [Codegen Mental Model](codegen-model.md) — terminals → matchers, profiles → guards, well-formedness.
- [Public API and Tooling](public-api.md) — the EBNF frontend CLI, the registry, and the embedding API.
- [Glossary](glossary.md).

## Authoritative sources behind this book

- `grammars/ebnf.ebnf` — the self-hosting EBNF meta-grammar (version `2.0`).
- `rust/src/ebnf_frontend.rs` — the Rust EBNF frontend that parses a `.ebnf` to raw AST JSON.
- `rust/src/ast_based_generator.rs`, `rust/src/ast_return_transform.rs` — the parser/return-shape codegen.
- `docs/RETURN_ANNOTATIONS_REFERENCE.md` and the
  [return_annotation book](../../return_annotation_parser_book/src/welcome.md) — the deep `-> …` detail.
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and the
  [semantic_annotation book](../../semantic_annotation_parser_book/src/welcome.md) — the deep `@…` detail.
- `docs/EBNF_INCLUDE_SYSTEM.md` — the include resolver.
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md` — the bootstrap path.
- The platform book chapters [Annotation System](../../book/src/annotation-system.md),
  [Lexical Annotations](../../book/src/lexical-annotations.md), and
  [Grammar Well-Formedness](../../book/src/grammar-wellformedness.md).
