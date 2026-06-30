# SEMANTIC-ANNOT-BOOK: a dedicated per-parser mdBook for the `semantic_annotation` parser

## Metadata

- Tree ID: `SEMANTIC-ANNOT-BOOK`
- Status: `active` (created + activated 2026-06-30)
- Family / slice-id prefix: `PGEN-SEMANTIC-ANNOT-BOOK-<NNNN>`
- Roadmap lane: Phase V — per-parser standalone integration mdBooks (one live book per generated
  parser, referenced from the platform book's Parser Families chapter)
- Created: `2026-06-30`
- Standing directive: director 2026-06-08 — every parser shall have its own per-parser mdBook. After
  `RETURN-ANNOT-BOOK.1` landed the `return_annotation` book, the README's still-to-come list is down to
  the `ebnf` meta-grammar (tree `EBNF-BOOK`) and this `semantic_annotation` grammar.

## Why

The `semantic_annotation` parser is the other core annotation grammar — the parser for the `@…`
**steering** language that makes PGEN parsers context-aware (store-gated rules, fact emission, scopes,
profiles, transforms, generation steering). It is a core platform grammar but had no per-parser
integration book, only the contract + the normative spec + the steering control matrix. Per the
books-are-the-user's-window doctrine ([[feedback_regex_book_live]]) and the every-parser-book directive
it should be a proper mdBook, gated like the others, paired with
`docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

Scope note: this is the parser-INTEGRATION reference for the `semantic_annotation` *parser* (the `@…`
surface it accepts, the parsed annotation envelope, the two backends, the embedding API). The deep
normative semantics live in `PGEN_ANNOTATION_NORMATIVE_SPEC.md` and the steering-capability matrix in
`PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`; the book curates + cross-links those, it does not duplicate
them. Complementary to the author-facing `EBNF-BOOK` lane.

## Goal

A curated, example-rich mdBook at `docs/semantic_annotation_parser_book/` (+ tracked
`docs/semantic_annotation_parser_book-html/` + `make -C rust SHELL=/bin/bash
semantic_annotation_parser_book_gate`) that is the authoritative integration reference for the
`semantic_annotation` parser, registered in the platform book's Parser Families chapter and the README.

## Authoritative sources (curated, not duplicated)

- `grammars/semantic_annotation.ebnf` (the `@…` language) and `grammars/builtin_semantic_annotation.ebnf`
  (the bootstrap-safe variant).
- `generated/semantic_annotation_parser.rs`, `generated/semantic_annotation.json`.
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` (downstream contract).
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  (deep normative detail behind the book).

## Task Tree

- ID: `SEMANTIC-ANNOT-BOOK`  Status: `active`
- ID: `.1`  Status: `done` (2026-06-30, `PGEN-SEMANTIC-ANNOT-BOOK-0001`)  Goal: author the
  complete `semantic_annotation` per-parser book (book.toml + SUMMARY + the chapter set), add the
  repo-standard gate (`semantic_annotation_parser_book_gate`) + Makefile target, register it in the
  platform book's Parser Families chapter + the README, build the tracked HTML, and verify the gate.

## Verification log

- `.1` (2026-06-30): authored 11 chapters (welcome, build-recipe, grammar-and-scope,
  values-and-references, steering-directives, semantic-store, ast-envelope, backends, public-api,
  schema-and-versioning, glossary) from `grammars/semantic_annotation.ebnf` (v2.0), the integration
  contract, and an agent digest of `PGEN_ANNOTATION_NORMATIVE_SPEC.md` +
  `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` (the SC-01…SC-13 steering catalog). `mdbook build
  docs/semantic_annotation_parser_book` PASS (47 HTML files); `make -C rust SHELL=/bin/bash
  semantic_annotation_parser_book_gate` GREEN; top-level `mdbook build docs/book` still PASS after the
  Parser Families registration edit. Pure-docs ⇒ code-change evidence gate N/A.

## Acceptance (PURE-DOCS lane — no `grammars/*.ebnf`, `rust/src/*`, `generated/*`, or
`ast_shape_contract/*.json` change; the code-change evidence checklist does not apply)

- `mdbook build docs/semantic_annotation_parser_book` succeeds; tracked HTML present.
- `make -C rust SHELL=/bin/bash semantic_annotation_parser_book_gate` is GREEN.
- Accurate against `grammars/semantic_annotation.ebnf` + the integration contract; no drift introduced.

## Blockers

- None.

## Changelog

- `2026-06-30`: tree created + activated and leaf `.1` DONE in the same slice
  (`PGEN-SEMANTIC-ANNOT-BOOK-0001`): authored the complete `semantic_annotation` per-parser book, wired
  the gate + Makefile target, registered it in the platform book + README, built the tracked HTML, and
  verified the gate + the top-level book build are green. Pure-docs; no
  code/grammar/generated/release/schema change. The every-parser-book directive now has only the `ebnf`
  meta-grammar book (tree `EBNF-BOOK`) still to come.
