# RTL-FE-MDBOOK: Stand-up rtl_frontend Parser mdBook

## Metadata

- Tree ID: `RTL-FE-MDBOOK`
- Status: `active`
- Roadmap lane: rtl_frontend deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
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
  Status: `done`
  Goal: `Author core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter references current grammar/contract paths.`
  Verification: `2026-05-15: 4 chapters authored; rtl_frontend_parser_book_gate passes.`
  Commit: `RTL-FE-MDBOOK-Slice-3`

- ID: `RTL-FE-MDBOOK.3`
  Status: `done`
  Goal: `Author shape-reference chapters (rules-top-level, json-carrier, walking-the-ast, schema-versioning) covering all 156 typed rules.`
  Acceptance: `Each chapter enumerates the typed shapes against the live inventory.`
  Verification: `2026-05-16: 4 chapters authored — rules-top-level (rtl_frontend_file root + 4-branch design_item dispatch + module_item/generate_item/package_item wrappers + 10 rule families + the 10-level binop_chain cascade), json-carrier, walking-the-ast (binop left-fold), schema-versioning (1.0.1 / schema v1, contract-aligned). Independently verified against generated/rtl_frontend_return_annotations.json: 156 annotations / 74 distinct rules; design_item=4, module_item=10, generate_item=11, package_item=3, binop_chain cascade = exactly 10 levels — all match. Contract 164-rules-vs-156-annotations nuance handled (chapters use the inventory-accurate "156 annotations on 74 distinct rules"). rtl_frontend_parser_book_gate green (independently re-run).`
  Commit: `RTL-FE-MDBOOK-Slice-4`

- ID: `RTL-FE-MDBOOK.4`
  Status: `pending`
  Goal: `Add the empty_module worked example with annotated AST dump.`
  Acceptance: `examples-empty-module.md exists with output validated against generated/rtl_frontend_parser.rs.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-MDBOOK.5`
  Status: `done`
  Goal: `Wire rtl_frontend_parser_book_gate target into Makefile + scripts.`
  Acceptance: `Makefile target exists; gate passes locally.`
  Verification: `2026-05-15: make rtl_frontend_parser_book_gate — pass.`
  Commit: `RTL-FE-MDBOOK-Slice-2`

- ID: `RTL-FE-MDBOOK.6`
  Status: `pending`
  Goal: `Wire glossary + changelog-index + link from README + LIVE_ACHIEVEMENT_STATUS.`
  Acceptance: `README and LIVE_ACHIEVEMENT_STATUS reference the book.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-FE-MDBOOK.4` | `pending` | The empty_module worked example (annotated AST dump validated against generated/rtl_frontend_parser.rs) comes next once the shape reference is in place. |
| 2 | `RTL-FE-MDBOOK.6` | `pending` | Glossary + changelog-index + README/LIVE_ACHIEVEMENT_STATUS links close the book — the final leaf. |

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
| `2026-05-15` | `RTL-FE-MDBOOK.5` | `make rtl_frontend_parser_book_gate` | `pass` |
| `2026-05-15` | `RTL-FE-MDBOOK.2` | `make rtl_frontend_parser_book_gate` | `pass — 4 chapters authored; gate green` |
| `2026-05-16` | `RTL-FE-MDBOOK.3` | `make rtl_frontend_parser_book_gate` + inventory cross-check | `pass — 4 shape-reference chapters; verified 156/74, design_item=4, module_item=10, generate_item=11, package_item=3, binop_chain=10 levels vs generated/rtl_frontend_return_annotations.json; gate green` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-MDBOOK.1` | `RTL-FE-MDBOOK-Slice-1` | book.toml + 12-entry SUMMARY + welcome + chapter stubs |
| `RTL-FE-MDBOOK.5` | `RTL-FE-MDBOOK-Slice-2` | gate script + Makefile target |
| `RTL-FE-MDBOOK.2` | `RTL-FE-MDBOOK-Slice-3` | quickstart + build-recipe + public-api + ast-envelope authored at SV parity |
| `RTL-FE-MDBOOK.3` | `RTL-FE-MDBOOK-Slice-4` | rules-top-level + json-carrier + walking-the-ast + schema-versioning authored; inventory-verified (156/74, 10-level binop_chain) |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `RTL-FE-MDBOOK.1` done; frontier → `.2` + `.5`.
- `2026-05-15`: `RTL-FE-MDBOOK.2` done; frontier advances to `.3` (shape-reference chapters).
- `2026-05-16`: `RTL-FE-MDBOOK.3` done; frontier advances to `.4` (empty_module worked example), then `.6` (glossary/changelog/links).
