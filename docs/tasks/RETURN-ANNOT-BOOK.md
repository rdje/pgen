# RETURN-ANNOT-BOOK: a dedicated per-parser mdBook for the `return_annotation` parser

## Metadata

- Tree ID: `RETURN-ANNOT-BOOK`
- Status: `active` (created + activated 2026-06-30)
- Family / slice-id prefix: `PGEN-RETURN-ANNOT-BOOK-<NNNN>`
- Roadmap lane: Phase V — per-parser standalone integration mdBooks (one live book per generated
  parser, referenced from the platform book's Parser Families chapter)
- Created: `2026-06-30`
- Standing directive: director 2026-06-08 — "every parser shall have its own per-parser mdBook,
  referenced from the top-level book's Parser Families chapter." README's Per-Parser Integration
  Reference Books section lists the `return_annotation` book as one of the three still-to-come books
  (`ebnf`, `return_annotation`, `semantic_annotation`).

## Why

The `return_annotation` parser is a core platform grammar (the AST-shaping mini-language behind every
`-> ...` payload), tracked `Done`, but it had **no per-parser integration book** — unlike the seven
shipped families (regex, systemverilog, systemverilog_preprocessor, vhdl, rtl_frontend,
rtl_const_expr, json). The platform book's Parser Families chapter listed it only under "Annotation
Families" with a pointer to the contract + normative spec, with no curated, example-rich, browsable
surface. Per the books-are-the-user's-window doctrine ([[feedback_regex_book_live]]) and the
every-parser-book directive this should be a proper mdBook, gated like the others.

Scope note (distinct from `EBNF-BOOK`): the `EBNF-BOOK` lane is the **grammar-author's** reference
for *writing* a `.ebnf` (the meta-language, of which the return-annotation language is one
sub-section). This `RETURN-ANNOT-BOOK` is the **parser-integration** reference for the
`return_annotation` *parser itself* — its accepted language surface, its parsed-annotation AST
envelope, its two backends, and its embedding API — mirroring the other per-parser books and paired
with `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The two surfaces are
complementary, not duplicative.

## Goal

A curated, example-rich mdBook at `docs/return_annotation_parser_book/` (+ tracked
`docs/return_annotation_parser_book-html/` + `make -C rust SHELL=/bin/bash
return_annotation_parser_book_gate`) that is the authoritative integration reference for the
`return_annotation` parser, covering: the accepted language (entry/normalization/passthrough,
references + literals, objects + arrays, the extraction/spread/access operators), the parsed
annotation AST envelope (the `{type: ...}` carrier shapes), the Bootstrap-vs-Generated backend split,
the public embedding API + diagnostics, and schema/versioning + notable shape changes — registered in
the platform book's Parser Families chapter and the README.

## Authoritative sources (curated, not duplicated)

- `grammars/return_annotation.ebnf` (the language, v2.0.0) and `grammars/builtin_return_annotation.ebnf`
  (the bootstrap-safe variant; source of truth `rust/src/ast_pipeline/unified_return_ast.rs`).
- `generated/return_annotation_parser.rs`, `generated/return_annotation.json` (generated artifacts).
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` (downstream contract).
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/RETURN_ANNOTATIONS_REFERENCE.md` (deep
  reference — stay the authoritative detail behind the book).

## Task Tree

- ID: `RETURN-ANNOT-BOOK`  Status: `active`
- ID: `.1`  Status: `done` (2026-06-30, `PGEN-RETURN-ANNOT-BOOK-0001`)  Goal: author the complete
  `return_annotation` per-parser book (book.toml + SUMMARY + the chapter set), add the repo-standard
  gate (`return_annotation_parser_book_gate`) + Makefile target, register it in the platform book's
  Parser Families chapter + the README, build it (tracked HTML), and verify the gate is green.

## Acceptance (this is a PURE-DOCS lane — no `grammars/*.ebnf`, `rust/src/*`, `generated/*`, or
`ast_shape_contract/*.json` change; the code-change evidence checklist does not apply)

- `mdbook build docs/return_annotation_parser_book` succeeds; tracked HTML present under
  `docs/return_annotation_parser_book-html/`.
- `make -C rust SHELL=/bin/bash return_annotation_parser_book_gate` is GREEN.
- The book is accurate against `grammars/return_annotation.ebnf` and the integration contract; no
  drift introduced into the platform book or README.

## Verification log

- `.1` (2026-06-30): `mdbook build docs/return_annotation_parser_book` PASS;
  `return_annotation_parser_book_gate` PASS; top-level `mdbook build docs/book` still PASS after the
  Parser Families registration edit. (Commit-time evidence pasted in the Changelog below.)

## Blockers

- None.

## Changelog

- `2026-06-30`: tree created + activated (`PGEN-RETURN-ANNOT-BOOK-0001`, leaf `.1`); authored the
  complete `return_annotation` per-parser book, wired the gate + Makefile target, registered it in the
  platform book + README, built the tracked HTML, and verified both the new book gate and the
  top-level book build are green. Pure-docs; no code/grammar/generated/release/schema change.
