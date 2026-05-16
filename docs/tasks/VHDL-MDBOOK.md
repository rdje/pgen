# VHDL-MDBOOK: Stand-up VHDL Parser mdBook

## Metadata

- Tree ID: `VHDL-MDBOOK`
- Status: `done`
- Roadmap lane: vhdl deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Bring the VHDL parser family up to the same documentation surface as the
SystemVerilog parser: a live `docs/vhdl_parser_book/` mdBook that downstream
consumers can read first, paired with a tracked HTML rendering and a
`make vhdl_parser_book_gate` target wired into the standard validation flow.

## Non-Goals

- Do not duplicate the VHDL LRM. The book is a parser-integration reference,
  not a language tutorial.
- Do not retrofit historical typing slices into chapter narratives. The book
  reflects the current AST shape, not the per-slice history.
- Do not block on full VHDL feature coverage. The current grammar
  (`grammars/vhdl.ebnf`, 449 lines, 249 annotations) is the slice baseline.

## Acceptance Criteria

- `docs/vhdl_parser_book/src/SUMMARY.md` lists at least: welcome, quickstart,
  public API, AST envelope, schema versioning, rules-top-level, JSON carrier,
  example for the `minimal_entity` sample, walking the AST, build recipe,
  changelog index, glossary.
- `docs/vhdl_parser_book/book.toml` builds via mdBook.
- `rust/Makefile` exposes `vhdl_parser_book_gate` mirroring the SV gate.
- The gate runs in CI-equivalent local invocation and produces tracked HTML
  under `docs/vhdl_parser_book-html/`.
- `README.md` and `LIVE_ACHIEVEMENT_STATUS.md` link the new book.

## Task Tree

- ID: `VHDL-MDBOOK`
  Status: `done`
  Goal: `Stand up the VHDL parser mdBook on parity with the SV book.`
  Children: `VHDL-MDBOOK.1`, `VHDL-MDBOOK.2`, `VHDL-MDBOOK.3`,
  `VHDL-MDBOOK.4`, `VHDL-MDBOOK.5`, `VHDL-MDBOOK.6`
  Result: `All 6 children done. The VHDL parser mdBook is fully stood up — scaffold + welcome, 4 core navigation chapters, 4 shape-reference chapters, the byte-verified minimal_entity worked example, the gate wiring, and the glossary + changelog-index closing leaf, with README + LIVE_ACHIEVEMENT_STATUS references. Tree complete 2026-05-16.`

- ID: `VHDL-MDBOOK.1`
  Status: `done`
  Goal: `Scaffold book.toml, SUMMARY.md, and the welcome chapter.`
  Acceptance: `book.toml + SUMMARY.md + welcome.md exist and mdbook build succeeds locally.`
  Verification: `2026-05-14: mdbook build wrote HTML to docs/vhdl_parser_book-html. SUMMARY has 12 entries; all stubbed.`
  Commit: `VHDL-MDBOOK-Slice-1`

- ID: `VHDL-MDBOOK.2`
  Status: `done`
  Goal: `Author the core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter exists and references current grammar/contract paths verbatim.`
  Verification: `2026-05-15: quickstart + build-recipe + public-api + ast-envelope authored; vhdl_parser_book_gate passes.`
  Commit: `VHDL-MDBOOK-Slice-3`

- ID: `VHDL-MDBOOK.3`
  Status: `done`
  Goal: `Author the shape-reference chapters (rules-top-level, json-carrier, walking-the-ast, schema-versioning).`
  Acceptance: `Each chapter enumerates the 249 typed rules' shapes against the live inventory.`
  Verification: `2026-05-16: 4 chapters authored — rules-top-level enumerates the vhdl_file root + 10-branch design_unit dispatch + all rule families grouped (249 annotations / 110 distinct rules), json-carrier, walking-the-ast (binop_chain left-fold), schema-versioning (1.0.1 / schema v1, contract-aligned). Content independently verified against generated/vhdl_return_annotations.json (branch counts, kinds, fields, binop_chain levels all match). vhdl_parser_book_gate passes (mdbook_build + tracked_html_check green).`
  Commit: `VHDL-MDBOOK-Slice-4`

- ID: `VHDL-MDBOOK.4`
  Status: `done`
  Goal: `Add the minimal_entity worked example with annotated AST dump.`
  Acceptance: `docs/vhdl_parser_book/src/examples-minimal-entity.md exists with the parsed output validated against generated/vhdl_parser.rs.`
  Verification: `2026-05-16: examples-minimal-entity.md authored with the REAL captured AST for the regression-locked input 'entity e is end e;\n' (probe ParseNode envelope + consumer AstDumpPayload.root view); doc JSON byte-identical to generated/vhdl_parser.rs output (python equality check). Surfaced + corrected a pre-existing accuracy defect in json-carrier.md (had claimed name was a bare string "e"; real shape is the un-annotated identifier envelope [[], "e"]). vhdl_parser_book_gate green.`
  Commit: `VHDL-MDBOOK-Slice-5`

- ID: `VHDL-MDBOOK.5`
  Status: `done`
  Goal: `Wire vhdl_parser_book_gate target into the Makefile + scripts.`
  Acceptance: `rust/Makefile exposes vhdl_parser_book_gate and rust/scripts/vhdl_parser_book_gate.sh exists; the gate passes locally.`
  Verification: `2026-05-15: make vhdl_parser_book_gate passes (mdbook_build + tracked_html_check).`
  Commit: `VHDL-MDBOOK-Slice-2`

- ID: `VHDL-MDBOOK.6`
  Status: `done`
  Goal: `Wire glossary + changelog-index and link the book from README + LIVE_ACHIEVEMENT_STATUS.`
  Acceptance: `Glossary and changelog-index chapters exist; README and LIVE_ACHIEVEMENT_STATUS reference the book and its gate.`
  Verification: `2026-05-16: glossary.md (15 terms, VHDL-correct, vhdl_v1.json manifest, single-batch VHDL-Slice-1 framing) + changelog-index.md (short by design — contract/ledger/CHANGES/git table + the 2 real release rows) authored and independently verified (links resolve, numbers 249/110/1.0.1/schema 1 correct). README.md gained a "Per-Parser Integration Reference Books" section listing all six books + gates; LIVE_ACHIEVEMENT_STATUS.md gained a Live Snapshot tracker note recording the completed VHDL-MDBOOK tree. vhdl_parser_book_gate green (independently re-run).`
  Commit: `VHDL-MDBOOK-Slice-6`

## Current Frontier

_None — the `VHDL-MDBOOK` tree is complete. All leaves `.1`–`.6` are `done`._

## Decisions

- `2026-05-14`: Mirror the existing SV parser book structure under
  `docs/systemverilog_parser_book/`. Re-using the layout reduces decision load
  per chapter and keeps cross-grammar mental model identical.

## Open Questions

- Which sample inputs beyond `minimal_entity` are worth promoting to worked
  examples? (entity + architecture, package + body, simple process). Not
  blocking the first frontier leaf.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-14` | `VHDL-MDBOOK.1` | `mdbook build` | `pass — HTML rendered to docs/vhdl_parser_book-html` |
| `2026-05-15` | `VHDL-MDBOOK.5` | `make vhdl_parser_book_gate` | `pass — mdbook_build + tracked_html_check both green` |
| `2026-05-15` | `VHDL-MDBOOK.2` | `make vhdl_parser_book_gate` | `pass — quickstart + build-recipe + public-api + ast-envelope authored; gate green` |
| `2026-05-16` | `VHDL-MDBOOK.3` | `make vhdl_parser_book_gate` + inventory cross-check | `pass — 4 shape-reference chapters authored; content verified against generated/vhdl_return_annotations.json (249/110); gate green` |
| `2026-05-16` | `VHDL-MDBOOK.4` | `make vhdl_parser_book_gate` + real-dump byte-equality check | `pass — examples-minimal-entity.md JSON byte-identical to generated/vhdl_parser.rs output; json-carrier.md accuracy defect corrected; gate green` |
| `2026-05-16` | `VHDL-MDBOOK.6` | `make vhdl_parser_book_gate` + link/number cross-check | `pass — glossary + changelog-index authored; README/LIVE references added; links resolve, numbers correct; gate green. Tree complete.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VHDL-MDBOOK.1` | `VHDL-MDBOOK-Slice-1` | book.toml + 12-entry SUMMARY + welcome chapter + stubs for remaining chapters |
| `VHDL-MDBOOK.5` | `VHDL-MDBOOK-Slice-2` | rust/scripts/vhdl_parser_book_gate.sh + Makefile target |
| `VHDL-MDBOOK.2` | `VHDL-MDBOOK-Slice-3` | quickstart + build-recipe + public-api + ast-envelope authored at SV parity |
| `VHDL-MDBOOK.3` | `VHDL-MDBOOK-Slice-4` | rules-top-level + json-carrier + walking-the-ast + schema-versioning authored; inventory-verified (249/110) |
| `VHDL-MDBOOK.4` | `VHDL-MDBOOK-Slice-5` | examples-minimal-entity.md (real captured AST, byte-verified) + json-carrier.md accuracy correction |
| `VHDL-MDBOOK.6` | `VHDL-MDBOOK-Slice-6` | glossary + changelog-index authored; README per-parser-books section + LIVE tracker note; **tree complete** |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `VHDL-MDBOOK.1` completed; frontier advances to `VHDL-MDBOOK.2` (next reader content) and `VHDL-MDBOOK.5` (gate wiring, parallel).
- `2026-05-15`: `VHDL-MDBOOK.2` completed; frontier advances to `VHDL-MDBOOK.3` (shape-reference chapters).
- `2026-05-16`: `VHDL-MDBOOK.3` completed; frontier advances to `VHDL-MDBOOK.4` (minimal_entity worked example), then `VHDL-MDBOOK.6` (glossary/changelog/links).
- `2026-05-16`: `VHDL-MDBOOK.4` completed (real captured AST, byte-verified; json-carrier.md accuracy defect corrected); frontier advances to `VHDL-MDBOOK.6` (final leaf — glossary/changelog/links).
- `2026-05-16`: `VHDL-MDBOOK.6` completed (glossary + changelog-index + README/LIVE references). All children `done`; **`VHDL-MDBOOK` tree closed** and moved to Completed Task Trees in `docs/TASK_TREE.md`.
