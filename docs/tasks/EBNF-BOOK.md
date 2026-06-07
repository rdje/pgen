# EBNF-BOOK: a dedicated EBNF-authoring / meta-grammar mdBook (the grammar-author's reference)

## Metadata

- Tree ID: `EBNF-BOOK`
- Status: `proposed` (owning skeleton — created 2026-06-07 per director; BUILD LATER, not PNT-eligible until
  activated)
- Family / slice-id prefix: `PGEN-EBNF-BOOK-<NNNN>`
- Roadmap lane: documentation — a first-class mdBook for the PGEN EBNF *language itself* (how to author a
  `.ebnf` grammar), parallel to the platform book and the per-parser books
- Created: `2026-06-07`
- Director directive: 2026-06-07 — "expose all the passthrough thing in the EBNF-specific book, if it
  exists; might be good to create one if it doesn't exist yet, but later — let's create a task-tree for that
  and own it later."

## Why

There is currently NO book documenting the PGEN EBNF *language* (terminals, rules, quantifiers, lookaheads,
the implicit/passthrough return policy, the return- and semantic-annotation languages, includes,
bootstrap). The meta-grammar (`grammars/ebnf.ebnf`) and codegen behaviours are documented only as code
comments + scattered reference docs (`docs/RETURN_ANNOTATIONS_REFERENCE.md`,
`docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/EBNF_INCLUDE_SYSTEM.md`) + per-feature task trees.
A grammar author has no single curated surface. Per the books-are-the-user's-window doctrine
([[feedback_regex_book_live]]) this should be a proper mdBook, gated like the others.

## Goal

A curated, example-rich mdBook (e.g. `docs/ebnf_book/` + tracked `docs/ebnf_book-html/` +
`make -C rust ebnf_book_gate`) that is the authoritative reference for writing a PGEN `.ebnf`, covering at
least:

- **Terminals**: literal strings `"..."`, char literals `'x'`, character classes `[...]` / negated
  `[^...]` / ranges `[a-z]`, regex literals `/.../` (and the REGEX-SELF-HOSTING guidance to prefer literals
  where Rust-regex-independence matters), the built-in **`any_char`** matcher (REGEX-SELF-HOSTING.2).
- **Rules / structure**: `rule := body`, ordered-choice `|`, sequences, grouping, quantifiers
  (`?`/`*`/`+`/`{n}`/`{n,m}` — the Layer-0 unified quantifier engine), negative/positive lookaheads
  (`!"x"` / `&"x"`).
- **The IMPLICIT / passthrough return policy** (the director's specific ask — full details captured below
  so they survive until the book is built).
- **Return-annotation language** (`-> ...`): positional `$N`, **`$0`/`$text` (whole-match text,
  REGEX-SELF-HOSTING.3)**, named `$name`, property `.field`, index `[i]`, spread `*` / flatten-spread `**`,
  objects/arrays/literals, extraction-spread `::N*`.
- **Semantic-annotation language** (`@...`): `@predicate` (+ `len_bounds`/`numeric_bounds`/`fact_*`),
  `@emit_fact`/`@open_scope`/`@export_to_library`/`@import_from_library`, `@profiles`, `@fact_kind:`,
  `@predicate_def:`, `@transform`, `@generate`, `@semantic_value`, `@dispatch_table`, etc. (the steering
  language) — bridging to `PGEN_ANNOTATION_NORMATIVE_SPEC.md`.
- **The `@include` system** (`docs/EBNF_INCLUDE_SYSTEM.md`).
- **The bootstrap path** (`builtin_*` annotation grammars; `docs/BOOTSTRAP_MODE_SPECIFICATION.md`).
- **Codegen mental model**: terminals → `match_string`/`match_regex`/`match_any_char`; profiles → runtime
  guards; well-formedness/lint (cross-link the Grammar Well-Formedness book chapter).

## Captured now (so it survives until the book is built) — the IMPLICIT / passthrough return policy

Tool-backed (2026-06-07, `ast_based_generator.rs:3701-3750` + `ast_return_transform.rs:679-691`):

- A rule/branch with **no explicit `-> ...`** gets a **codegen-only synthetic `-> $1`** ("implicit
  passthrough") **iff its body is a SINGLE element** — i.e. `body_has_single_element`: a `Sequence` whose
  `elements.len() <= 1`, OR a lone terminal / rule-reference / lookahead / `Or`.
- It is **deliberately NOT** synthesized for:
  - **multi-element Sequences (≥2)** — an implicit `$1` would silently drop every element past the first
    (e.g. `'(' expr ')'` → `$1` = `'('`, not the payload);
  - **Quantified bodies** (`+`/`*`/`?`) — left as **raw passthrough**; the "natural reading" of `$1` on a
    Quantified is "the whole capture group", and raw passthrough already yields that — BUT raw passthrough
    of a Quantified yields the **structure**, not the matched text (this is exactly why quantified
    text-payloads historically used `/.../`; `$0`/`$text` from REGEX-SELF-HOSTING.3 is the native fix).
- `generate_passthrough` (the no-transform default): empty captures → `Terminal("")`; 1 capture → that
  capture (the implicit `$1`); N captures → the **last** capture.
- The synthetic `-> $1` is codegen-only — it never appears in the return-annotation inventory artifact
  (which surfaces only author-written annotations).
- **Future clarification (REGEX-SELF-HOSTING "eventually")**: extend the policy so a terminal-only
  Quantified body with no `->` defaults to `$0`/`$text` (matched text) instead of raw structure — making
  the implicit model {single-element→`$1`, terminal-Quantified→`$0`-text} principled and removing most
  explicit `-> $text`. Behaviour change → verify oracle byte-identical.

## Task Tree

- ID: `EBNF-BOOK`  Status: `proposed`
- ID: `.1`  Status: `pending`  Goal: SCOPING — confirm the book skeleton (`book.toml` + `SUMMARY.md` +
  chapter list), the gate (`ebnf_book_gate`), and the curation split vs the existing reference docs
  (`RETURN_ANNOTATIONS_REFERENCE`, `PGEN_ANNOTATION_NORMATIVE_SPEC`, `EBNF_INCLUDE_SYSTEM`,
  `BOOTSTRAP_MODE_SPECIFICATION`) — the book curates + cross-links, the reference docs stay the deep detail.
- ID: `.2`+  Status: `pending`  Goal: author the chapters (terminals, structure/quantifiers/lookaheads,
  the passthrough/implicit-return policy [above], `$0`/`$text`, `any_char`, return-/semantic-annotation
  languages, includes, bootstrap, codegen mental model), with worked examples; wire the gate; track HTML.

## Blockers

- None — backlog. Activate when prioritized (the director said "own it later"). The REGEX-SELF-HOSTING
  `$text`/`$0`/`any_char` features should land first so the book documents the final surface.

## Changelog

- `2026-06-07`: tree created as an owning skeleton (`PGEN-EBNF-BOOK-0001`) per the director — build the EBNF
  book later; captured the passthrough/implicit-return policy + the planned chapter coverage now so the
  knowledge is durable.
