# RTL-CE-MDBOOK: Stand-up rtl_const_expr Parser mdBook

## Metadata

- Tree ID: `RTL-CE-MDBOOK`
- Status: `active`
- Roadmap lane: rtl_const_expr deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
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
  Status: `done`
  Goal: `Author core navigation chapters (quickstart, public-api, ast-envelope, build-recipe).`
  Acceptance: `Each chapter references current grammar/contract paths.`
  Verification: `2026-05-15: 4 chapters authored; rtl_const_expr_parser_book_gate passes.`
  Commit: `RTL-CE-MDBOOK-Slice-3`

- ID: `RTL-CE-MDBOOK.3`
  Status: `done`
  Goal: `Author shape-reference chapters covering all 24 typed rules including the binop_chain shape.`
  Acceptance: `Each chapter enumerates typed shapes; binop_chain documented as the consumer-facing left-fold contract.`
  Verification: `2026-05-16: 4 shape-reference chapters authored (rules-top-level, json-carrier, walking-the-ast, schema-versioning), independently verified vs generated/rtl_const_expr_return_annotations.json — 24 annotations / 16 distinct rules, root {type:rtl_const_expr,expr}, conditional_expr ternary+passthrough, 10-level binop_chain (rest=$2 raw envelope), unary 4 typed + passthrough, literal 2 kinds, identifier — all match. ALSO corrected a confirmed accuracy defect in the pre-existing sibling chapter ast-envelope.md (RTL-CE-MDBOOK-Slice-3 / 8ffcd78d, pushed): it claimed {kind:"ternary"}/{kind:"plus"}/typed {op,rhs} rest / "10-annotation surface" — rewritten to the live shapes so the book is internally consistent. rtl_const_expr_parser_book_gate green (independently re-run).`
  Commit: `RTL-CE-MDBOOK-Slice-4`

- ID: `RTL-CE-MDBOOK.4`
  Status: `done`
  Goal: `Add literal_42 and binary_addition worked examples with annotated AST dumps.`
  Acceptance: `examples-*.md exist; outputs validated against generated/rtl_const_expr_parser.rs.`
  Verification: `2026-05-16: examples-literal-42.md + examples-binary-addition.md authored from the REAL captured AST of the FIXED parser (post RTL-CE-Slice-2/PGEN-RTL-0002 — implementing this leaf is what surfaced the 3 correctness bugs). Doc JSON independently re-verified vs /tmp dumps: literal_42 = 10-level binop_chain stack bottoming at clean {kind:decimal,text:"42"}; binary_addition additive.rest[0] = [["","+"], {operand id b}] (op text at [0][1]), identifier text clean. rtl_const_expr_parser_book_gate green; links resolve; no stub markers.`
  Commit: `RTL-CE-MDBOOK-Slice-5`

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
| 1 | `RTL-CE-MDBOOK.6` | `pending` | Glossary + changelog-index + README/LIVE links close the book — final leaf. |

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
| `2026-05-15` | `RTL-CE-MDBOOK.2` | `make rtl_const_expr_parser_book_gate` | `pass — 4 chapters authored; gate green` |
| `2026-05-16` | `RTL-CE-MDBOOK.3` | `make rtl_const_expr_parser_book_gate` + inventory cross-check | `pass — 4 shape-reference chapters; verified 24/16, root/ternary/10-level binop_chain/unary/literal/identifier vs generated/rtl_const_expr_return_annotations.json; corrected confirmed ast-envelope.md inaccuracy (kind→type, 10→24, typed-rest→raw-envelope); gate green` |
| `2026-05-16` | `RTL-CE-MDBOOK.4` | `make rtl_const_expr_parser_book_gate` + real-dump faithfulness check | `pass — literal_42 + binary_addition examples authored from FIXED parser (post PGEN-RTL-0002); doc JSON re-verified vs /tmp dumps (10-level stack→clean literal "42"; additive.rest[0]=[["","+"],id b]); gate green` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-CE-MDBOOK.1` | `RTL-CE-MDBOOK-Slice-1` | book.toml + 13-entry SUMMARY + welcome + chapter stubs |
| `RTL-CE-MDBOOK.5` | `RTL-CE-MDBOOK-Slice-2` | gate script + Makefile target |
| `RTL-CE-MDBOOK.2` | `RTL-CE-MDBOOK-Slice-3` | quickstart + build-recipe + public-api + ast-envelope authored at SV parity |
| `RTL-CE-MDBOOK.3` | `RTL-CE-MDBOOK-Slice-4` | 4 shape-reference chapters (24/16, 10-level binop_chain) + ast-envelope.md accuracy correction |
| `RTL-CE-MDBOOK.4` | `RTL-CE-MDBOOK-Slice-5` | literal_42 + binary_addition worked examples from the FIXED parser (real captured AST, byte-verified); surfaced PGEN-RTL-0002 |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-14`: `.1` done; frontier → `.2` + `.5`.
- `2026-05-15`: `.2` done; frontier advances to `.3` (shape-reference chapters).
- `2026-05-16`: `.3` done (4 shape chapters + ast-envelope.md accuracy correction); frontier advances to `.4` (worked examples), then `.6` (glossary/changelog/links).
- `2026-05-16`: while implementing `.4`, the worked examples surfaced 3 real `rtl_const_expr` parser-correctness bugs (binop_chain `<invalid_sequence_access>`, empty `identifier.text`, envelope `literal.text`). Fixed under RTL-CE-Slice-2 / `PGEN-RTL-0002` (parser regen + manifest tighten + contract `1.0.2`/schema `2`); all 5 `.3` book chapters re-synced to the corrected 26-annotation shape in lockstep. `.4` resumes against the now-clean parser output.
- `2026-05-16`: `.4` done (literal_42 + binary_addition worked examples authored from the fixed parser, byte-verified); frontier advances to `.6` (final leaf — glossary/changelog/links).
