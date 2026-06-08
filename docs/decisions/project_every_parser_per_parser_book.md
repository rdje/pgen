---
name: project-every-parser-per-parser-book
description: STANDING DIRECTIVE (director 2026-06-08) — EVERY PGEN parser shall have its OWN per-parser mdBook (the canonical AST-integration reference for that parser), and each such book SHALL be referenced/linked from the top-level platform book (docs/book/, the Parser Families chapter). Both the src/*.md and the rendered *-html/ are tracked in git for GitHub browsability. As of 2026-06-08: regex/SV/SVPP/vhdl/rtl_frontend/rtl_const_expr/json have books; ebnf + return_annotation + semantic_annotation still need one.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-08
---

**THE DIRECTIVE (binding, director 2026-06-08).** "Every parser shall have its parser-specific mdBook
which shall be referenced from the top-level mdBook." So:

1. **Every PGEN parser has its own per-parser mdBook** — a self-contained `docs/<parser>_parser_book/`
   mdBook that is the canonical AST-integration reference for that parser (welcome/scope, build recipe,
   grammar scope, AST envelope, quality/characterization, glossary; for shipped families also the
   per-construct worked examples + per-release changelog).
2. **Each per-parser book is referenced from the top-level platform book** (`docs/book/`) — specifically
   the **Parser Families** chapter's "Per-Parser Integration Reference Books" table (added by
   `PGEN-BOOK-XLINK-0001`), with repo-relative `../../<parser>_parser_book/src/welcome.md` links that
   resolve when browsing the repo on GitHub.
3. Both the `src/*.md` source AND the rendered `../<parser>_parser_book-html/` are tracked in git (so the
   book is browsable on GitHub without an mdbook install), and each book has a repo-standard gate
   `make -C rust SHELL=/bin/bash <parser>_parser_book_gate` (required files exist + `mdbook build`).

**Why.** The book is the director's window into the project ([[feedback_regex_book_live]]); a per-parser
book is the precise, browsable contract for each parser's behaviour and AST. "Every parser" makes the
coverage complete and uniform — no parser is undocumented.

**Status (2026-06-08).** Books exist for: `regex`, `systemverilog`, `systemverilog_preprocessor`, `vhdl`,
`rtl_frontend`, `rtl_const_expr`, and **`json`** (`PGEN-BOOK-JSON-0001` — honestly framed as a *simplified
built-in* grammar, paired with `json_corpus_bundle/` instead of a downstream contract). **Still missing
(follow-up slices):** `ebnf` (the meta-grammar), `return_annotation`, `semantic_annotation` (the built-in
annotation variants can share/redirect). Each new parser is "documented" only once it has its book + the
top-level link.

**How to apply.** When a parser is added or matured, create its `docs/<parser>_parser_book/` (mirror
`docs/regex_parser_book/` structure), render `../<parser>_parser_book-html/`, add a
`<parser>_parser_book_gate` (mirror `rust/scripts/json_parser_book_gate.sh`), and add a row to the
top-level book's Parser Families "Per-Parser Integration Reference Books" table + the README list.
Composes with [[feedback_regex_book_live]] (book↔codebase lockstep) and the EBNF-BOOK proposed tree (the
EBNF-authoring book is a *separate* surface from the per-parser `ebnf` integration book).
