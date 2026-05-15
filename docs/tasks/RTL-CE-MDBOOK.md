# RTL-CE-MDBOOK: Stand-up rtl_const_expr Parser mdBook

## Metadata

- Tree ID: `RTL-CE-MDBOOK`
- Status: `active`
- Roadmap lane: rtl_const_expr deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Bring the rtl_const_expr parser family up to the same documentation surface
as the SystemVerilog parser: a live `docs/rtl_const_expr_parser_book/` mdBook
covering the constant-expression subset, the binop_chain shape, and the
worked literal_42 / binary_addition examples already tracked in the manifest.

## Non-Goals

- Do not duplicate the rtl_frontend book — these are separate grammars with
  different scopes.
- Do not document features outside rtl_const_expr.ebnf (no statements, no
  control flow).

## Acceptance Criteria

- `docs/rtl_const_expr_parser_book/src/SUMMARY.md` covers welcome,
  quickstart, public API, AST envelope, schema versioning, rules-top-level,
  JSON carrier, walking the AST, examples (literal_42, binary_addition),
  build recipe, changelog index, glossary.
- mdBook builds; gate target exists; HTML tracked.

## Task Tree

- ID: `RTL-CE-MDBOOK`
  Status: `active`
  Goal: `Stand up the rtl_const_expr parser mdBook.`
  Children: `RTL-CE-MDBOOK.1`, `RTL-CE-MDBOOK.2`, `RTL-CE-MDBOOK.3`,
  `RTL-CE-MDBOOK.4`, `RTL-CE-MDBOOK.5`, `RTL-CE-MDBOOK.6`

- ID: `RTL-CE-MDBOOK.1`
  Status: `done`
  Goal: `Scaffold book.toml, SUMMARY.md, welcome chapter.`
  Acceptance: `mdbook build succeeds locally.`
  Verification: `2026-05-14: mdbook build wrote HTML to docs/rtl_const_expr_parser_book-html.`
  Commit: `RTL-CE-MDBOOK-Slice-1`

- ID: `RTL-CE-MDBOOK.2`
  Status: `pending`
  Goal: `Author core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter references current grammar/contract paths.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-CE-MDBOOK.3`
  Status: `pending`
  Goal: `Author shape-reference chapters covering all 24 typed rules including the binop_chain shape.`
  Acceptance: `Each chapter enumerates typed shapes; binop_chain documented as the consumer-facing left-fold contract.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-CE-MDBOOK.4`
  Status: `pending`
  Goal: `Add literal_42 and binary_addition worked examples with annotated AST dumps.`
  Acceptance: `examples-*.md exist; outputs validated against generated/rtl_const_expr_parser.rs.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-CE-MDBOOK.5`
  Status: `done`
  Goal: `Wire rtl_const_expr_parser_book_gate Makefile target.`
  Acceptance: `Gate passes locally; HTML tracked.`
  Verification: `2026-05-15: make rtl_const_expr_parser_book_gate — pass.`
  Commit: `RTL-CE-MDBOOK-Slice-2`

- ID: `RTL-CE-MDBOOK.6`
  Status: `pending`
  Goal: `Wire glossary, changelog-index, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `README and LIVE_ACHIEVEMENT_STATUS reference the book.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-CE-MDBOOK.2` | `pending` | Core navigation content. |

## Decisions

- `2026-05-14`: Document the binop_chain shape as the consumer-facing contract
  for all 10 binary-operator chain rules; consumers fold left.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-14` | `RTL-CE-MDBOOK.1` | `mdbook build` | `pass` |
| `2026-05-15` | `RTL-CE-MDBOOK.5` | `make rtl_const_expr_parser_book_gate` | `pass` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-CE-MDBOOK.1` | `RTL-CE-MDBOOK-Slice-1` | book.toml + 13-entry SUMMARY + welcome + chapter stubs |
| `RTL-CE-MDBOOK.5` | `RTL-CE-MDBOOK-Slice-2` | gate script + Makefile target |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `.1` done; frontier → `.2` + `.5`.
