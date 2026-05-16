# SVPP-MDBOOK: Stand-up sv_preprocessor Parser mdBook

## Metadata

- Tree ID: `SVPP-MDBOOK`
- Status: `done`
- Roadmap lane: sv_preprocessor deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Bring the systemverilog_preprocessor parser family up to the same
documentation surface as the SystemVerilog parser: a live
`docs/systemverilog_preprocessor_parser_book/` mdBook covering directive
parsing, conditional compilation, macro definitions, and the line-oriented
shape conventions.

## Non-Goals

- Do not duplicate the full SystemVerilog mdBook. The preprocessor is a
  separate parser family with a distinct shape surface.
- Do not document features outside the current preprocessor grammar scope.

## Acceptance Criteria

- `docs/systemverilog_preprocessor_parser_book/src/SUMMARY.md` covers
  welcome, quickstart, public API, AST envelope, schema versioning,
  rules-top-level, JSON carrier, walking the AST, single_define example,
  build recipe, changelog index, glossary.
- mdBook builds; gate target exists; HTML tracked.

## Task Tree

- ID: `SVPP-MDBOOK`
  Status: `done`
  Goal: `Stand up the systemverilog_preprocessor parser mdBook.`
  Children: `SVPP-MDBOOK.1`, `SVPP-MDBOOK.2`, `SVPP-MDBOOK.3`,
  `SVPP-MDBOOK.4`, `SVPP-MDBOOK.5`, `SVPP-MDBOOK.6`
  Result: `All 6 children done. The sv_preprocessor parser mdBook is fully stood up (release 1.0.1 / schema 1 / 64 annotations / 27 distinct rules). The .4 conditional worked example surfaced released-parser defect SVPP-0001 (pp_if_branch.keyword <invalid_sequence_access>, inline-alternation-$N class) — documented honestly book-wide + contract Known Defects + bug ledger; parser fix deferred to the tracked systemic inline-alternation correctness lane. Tree complete 2026-05-16; 4th completed tree.`

- ID: `SVPP-MDBOOK.1`
  Status: `done`
  Goal: `Scaffold book.toml, SUMMARY.md, welcome chapter.`
  Acceptance: `mdbook build succeeds locally.`
  Verification: `2026-05-14: mdbook build wrote HTML to docs/systemverilog_preprocessor_parser_book-html.`
  Commit: `SVPP-MDBOOK-Slice-1`

- ID: `SVPP-MDBOOK.2`
  Status: `done`
  Goal: `Author core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter references current grammar/contract paths.`
  Verification: `2026-05-15: quickstart + build-recipe + public-api + ast-envelope authored; systemverilog_preprocessor_parser_book_gate passes (mdbook_build + tracked_html_check both green).`
  Commit: `SVPP-MDBOOK-Slice-3`

- ID: `SVPP-MDBOOK.3`
  Status: `done`
  Goal: `Author shape-reference chapters covering all 64 typed rules including pp_item dispatch, macro fragments, condition_atom.`
  Acceptance: `Each chapter enumerates typed shapes against the live inventory.`
  Verification: `2026-05-16: 4 shape-reference chapters authored (rules-top-level, json-carrier, walking-the-ast, schema-versioning) — independently verified vs generated/systemverilog_preprocessor_return_annotations.json: 64 annotations / 27 distinct rules; pp_item 10 kinds, condition_atom 12, macro_body_fragment 9, macro_default_atom 8 all match; single_define real AST captured. Book-wide stale-content audit also reconciled 2 pre-existing sibling defects: quickstart.md transposed fn name (parse_grammar_profile_named_ast_dump -> parse_grammar_profile_ast_dump_named) and ast-envelope.md invented macro_body_fragment kind names (-> real token_paste/stringize/macro_reference/text/lparen/rparen/comma/question/colon). systemverilog_preprocessor_parser_book_gate green.`
  Commit: `SVPP-MDBOOK-Slice-4`

- ID: `SVPP-MDBOOK.4`
  Status: `done`
  Goal: `Add the single_define worked example plus a conditional-compilation example.`
  Acceptance: `examples-*.md exist; outputs validated against generated/systemverilog_preprocessor_parser.rs.`
  Verification: `2026-05-16: examples-single-define.md + examples-conditional.md authored from the REAL captured AST. single_define independently re-verified clean (no <invalid_sequence_access>). The conditional example surfaced a real released-parser defect — pp_if_branch.keyword <invalid_sequence_access> (same inline-alternation-$N root cause as RTL-CE-Slice-2); documented HONESTLY in examples-conditional.md (dedicated section + safe consumer workaround), filed as ledger SVPP-0001 (Root Caused), and a Known Defects note added to the sv_preprocessor contract. Parser NOT fixed in this leaf — it joins the tracked systemic inline-alternation correctness lane (rtl_frontend + vhdl + sv_preprocessor). systemverilog_preprocessor_parser_book_gate green.`
  Commit: `SVPP-MDBOOK-Slice-5`

- ID: `SVPP-MDBOOK.5`
  Status: `done`
  Goal: `Wire systemverilog_preprocessor_parser_book_gate Makefile target.`
  Acceptance: `Gate passes locally; HTML tracked.`
  Verification: `2026-05-15: make systemverilog_preprocessor_parser_book_gate — pass.`
  Commit: `SVPP-MDBOOK-Slice-2`

- ID: `SVPP-MDBOOK.6`
  Status: `done`
  Goal: `Wire glossary, changelog-index, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `README and LIVE_ACHIEVEMENT_STATUS reference the book.`
  Verification: `2026-05-16: glossary.md (21 terms, incl. SVPP-0001 known-defect entry) + changelog-index.md (2 release rows + Known defects section) authored; independently verified 64/27, links resolve, SVPP-0001 reflected honestly, no stub. README already lists the book + gate (Per-Parser Integration Reference Books section, no edit needed); LIVE_ACHIEVEMENT_STATUS tracker note added. systemverilog_preprocessor_parser_book_gate green (independently re-run). Verified 63-vs-64 framing reconciles (63 new + 1 baseline root = 64; canonical 64 used throughout the book) — no action needed.`
  Commit: `SVPP-MDBOOK-Slice-6`

## Current Frontier

_None — the `SVPP-MDBOOK` tree is complete. All leaves `.1`–`.6` are `done`._

## Decisions

- `2026-05-14`: Document the pp_item dispatch (10 kinds) and macro_body
  fragment shape (9 kinds) as the consumer-facing tour points.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-14` | `SVPP-MDBOOK.1` | `mdbook build` | `pass` |
| `2026-05-15` | `SVPP-MDBOOK.5` | `make systemverilog_preprocessor_parser_book_gate` | `pass` |
| `2026-05-15` | `SVPP-MDBOOK.2` | `make systemverilog_preprocessor_parser_book_gate` | `pass — quickstart + build-recipe + public-api + ast-envelope authored; mdbook_build + tracked_html_check both green` |
| `2026-05-16` | `SVPP-MDBOOK.3` | `make systemverilog_preprocessor_parser_book_gate` + inventory cross-check + book-wide stale audit | `pass — 4 shape-reference chapters; verified 64/27, pp_item=10/condition_atom=12/macro_body_fragment=9/macro_default_atom=8; reconciled 2 stale .2-era sibling defects (quickstart fn name, ast-envelope invented fragment kinds); gate green` |
| `2026-05-16` | `SVPP-MDBOOK.4` | `make systemverilog_preprocessor_parser_book_gate` + real-dump capture + defect triage | `pass — single_define + conditional examples from real AST; single_define clean; conditional surfaced + honestly documented the SVPP-0001 pp_if_branch <invalid_sequence_access> defect (ledger + contract note added); gate green` |
| `2026-05-16` | `SVPP-MDBOOK.6` | `make systemverilog_preprocessor_parser_book_gate` + counts/links/SVPP-0001 cross-check | `pass — glossary + changelog-index authored (64/27 verified, SVPP-0001 reflected honestly); README already lists book; LIVE tracker note added; 63-vs-64 framing verified consistent; gate green. Tree complete.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SVPP-MDBOOK.1` | `SVPP-MDBOOK-Slice-1` | book.toml + 13-entry SUMMARY + welcome + chapter stubs |
| `SVPP-MDBOOK.5` | `SVPP-MDBOOK-Slice-2` | gate script + Makefile target |
| `SVPP-MDBOOK.2` | `SVPP-MDBOOK-Slice-3` | quickstart + build-recipe + public-api + ast-envelope authored at SV parity |
| `SVPP-MDBOOK.3` | `SVPP-MDBOOK-Slice-4` | 4 shape-reference chapters (64/27, pp_item/condition_atom/macro fragments) + 2 stale-sibling reconciliations |
| `SVPP-MDBOOK.4` | `SVPP-MDBOOK-Slice-5` | single_define + conditional worked examples (real AST); surfaced + tracked SVPP-0001 (ledger + contract note) |
| `SVPP-MDBOOK.6` | `SVPP-MDBOOK-Slice-6` | glossary + changelog-index (SVPP-0001 reflected); LIVE tracker note; **tree complete** |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `.1` done; frontier → `.2` + `.5`.
- `2026-05-15`: `.2` completed; frontier advances to `.3` (shape-reference chapters), then `.4` (worked examples), then `.6` (glossary/changelog/links).
- `2026-05-16`: `.3` completed (4 shape chapters, 64/27 inventory-verified; reconciled 2 stale `.2`-era sibling defects); frontier advances to `.4` (worked examples), then `.6` (glossary/changelog/links).
- `2026-05-16`: `.4` completed (single_define + conditional worked examples from real AST). The conditional example surfaced released-parser defect `SVPP-0001` (`pp_if_branch.keyword` `<invalid_sequence_access>`, inline-alternation-`$N` class); documented honestly + filed to the bug ledger + contract Known Defects note; parser fix deferred to the tracked systemic inline-alternation correctness lane (rtl_frontend + vhdl + sv_preprocessor). Frontier advances to `.6` (final leaf — glossary/changelog/links).
- `2026-05-16`: `.6` completed (glossary + changelog-index, SVPP-0001 reflected; LIVE tracker note; README already covered). All children `done`; **`SVPP-MDBOOK` tree closed** and moved to Completed Task Trees in `docs/TASK_TREE.md` — 4th completed tree. All four per-parser mdBook stand-up trees (VHDL/RTL-FE/RTL-CE/SVPP) are now complete.
