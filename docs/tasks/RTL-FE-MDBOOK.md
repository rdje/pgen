# RTL-FE-MDBOOK: Stand-up rtl_frontend Parser mdBook

## Metadata

- Tree ID: `RTL-FE-MDBOOK`
- Status: `active`
- Roadmap lane: rtl_frontend deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Bring the rtl_frontend parser family up to the same documentation surface as
the SystemVerilog parser: a live `docs/rtl_frontend_parser_book/` mdBook
that downstream consumers (RTLSyn) can read first, paired with a tracked HTML
rendering and an `rtl_frontend_parser_book_gate` target.

## Non-Goals

- Do not document RTL features outside the current rtl_frontend.ebnf scope
  (e.g. SystemVerilog interfaces, full SV expressions).
- Do not retrofit historical RTL-FE-Slice-1..7 narrative into the book.

## Acceptance Criteria

- `docs/rtl_frontend_parser_book/src/SUMMARY.md` covers welcome, quickstart,
  public API, AST envelope, schema versioning, rules-top-level, JSON carrier,
  worked-example (empty_module), walking the AST, build recipe, changelog
  index, glossary.
- mdBook builds; gate target exists and passes; HTML tracked under
  `docs/rtl_frontend_parser_book-html/`.

## Task Tree

- ID: `RTL-FE-MDBOOK`
  Status: `active`
  Goal: `Stand up the rtl_frontend parser mdBook on parity with the SV book.`
  Children: `RTL-FE-MDBOOK.1`, `RTL-FE-MDBOOK.2`, `RTL-FE-MDBOOK.3`,
  `RTL-FE-MDBOOK.4`, `RTL-FE-MDBOOK.5`, `RTL-FE-MDBOOK.6`

- ID: `RTL-FE-MDBOOK.1`
  Status: `done`
  Goal: `Scaffold book.toml, SUMMARY.md, welcome chapter.`
  Acceptance: `mdbook build succeeds locally.`
  Verification: `2026-05-14: mdbook build wrote HTML to docs/rtl_frontend_parser_book-html.`
  Commit: `RTL-FE-MDBOOK-Slice-1`

- ID: `RTL-FE-MDBOOK.2`
  Status: `pending`
  Goal: `Author core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter references current grammar/contract paths.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-MDBOOK.3`
  Status: `pending`
  Goal: `Author shape-reference chapters (rules-top-level, json-carrier, walking-the-ast, schema-versioning) covering all 156 typed rules.`
  Acceptance: `Each chapter enumerates the typed shapes against the live inventory.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-MDBOOK.4`
  Status: `pending`
  Goal: `Add the empty_module worked example with annotated AST dump.`
  Acceptance: `examples-empty-module.md exists with output validated against generated/rtl_frontend_parser.rs.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-MDBOOK.5`
  Status: `pending`
  Goal: `Wire rtl_frontend_parser_book_gate target into Makefile + scripts.`
  Acceptance: `Makefile target exists; gate passes locally.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-MDBOOK.6`
  Status: `pending`
  Goal: `Wire glossary + changelog-index + link from README + LIVE_ACHIEVEMENT_STATUS.`
  Acceptance: `README and LIVE_ACHIEVEMENT_STATUS reference the book.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-FE-MDBOOK.2` | `pending` | Core navigation chapters next reader entry point. |
| 2 | `RTL-FE-MDBOOK.5` | `pending` | Gate wiring infrastructure (parallel). |

## Decisions

- `2026-05-14`: Mirror the SV parser book structure for cross-grammar
  consistency.

## Open Questions

- Should the rtl_frontend book share chapters with rtl_const_expr (same crate
  ecosystem, similar shapes) or stay separate? Defer until both books have
  their respective frontiers cleared.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-14` | `RTL-FE-MDBOOK.1` | `mdbook build` | `pass` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-MDBOOK.1` | `RTL-FE-MDBOOK-Slice-1` | book.toml + 12-entry SUMMARY + welcome + chapter stubs |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `RTL-FE-MDBOOK.1` done; frontier → `.2` + `.5`.
