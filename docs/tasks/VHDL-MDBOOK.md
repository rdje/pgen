# VHDL-MDBOOK: Stand-up VHDL Parser mdBook

## Metadata

- Tree ID: `VHDL-MDBOOK`
- Status: `active`
- Roadmap lane: vhdl deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
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
  Status: `active`
  Goal: `Stand up the VHDL parser mdBook on parity with the SV book.`
  Children: `VHDL-MDBOOK.1`, `VHDL-MDBOOK.2`, `VHDL-MDBOOK.3`,
  `VHDL-MDBOOK.4`, `VHDL-MDBOOK.5`, `VHDL-MDBOOK.6`

- ID: `VHDL-MDBOOK.1`
  Status: `pending`
  Goal: `Scaffold book.toml, SUMMARY.md, and the welcome chapter.`
  Acceptance: `book.toml + SUMMARY.md + welcome.md exist and mdbook build succeeds locally.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-MDBOOK.2`
  Status: `pending`
  Goal: `Author the core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter exists and references current grammar/contract paths verbatim.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-MDBOOK.3`
  Status: `pending`
  Goal: `Author the shape-reference chapters (rules-top-level, json-carrier, walking-the-ast, schema-versioning).`
  Acceptance: `Each chapter enumerates the 249 typed rules' shapes against the live inventory.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-MDBOOK.4`
  Status: `pending`
  Goal: `Add the minimal_entity worked example with annotated AST dump.`
  Acceptance: `docs/vhdl_parser_book/src/examples-minimal-entity.md exists with the parsed output validated against generated/vhdl_parser.rs.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-MDBOOK.5`
  Status: `pending`
  Goal: `Wire vhdl_parser_book_gate target into the Makefile + scripts.`
  Acceptance: `rust/Makefile exposes vhdl_parser_book_gate and rust/scripts/vhdl_parser_book_gate.sh exists; the gate passes locally.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-MDBOOK.6`
  Status: `pending`
  Goal: `Wire glossary + changelog-index and link the book from README + LIVE_ACHIEVEMENT_STATUS.`
  Acceptance: `Glossary and changelog-index chapters exist; README and LIVE_ACHIEVEMENT_STATUS reference the book and its gate.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `VHDL-MDBOOK.1` | `pending` | Scaffolding must exist before any chapter can be authored. |

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
| `2026-05-14` | `VHDL-MDBOOK.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VHDL-MDBOOK.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree.
