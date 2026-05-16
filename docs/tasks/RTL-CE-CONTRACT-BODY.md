# RTL-CE-CONTRACT-BODY: Bring rtl_const_expr contract to SV parity

## Metadata

- Tree ID: `RTL-CE-CONTRACT-BODY`
- Status: `done`
- Roadmap lane: rtl_const_expr deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

There is no rtl_const_expr integration contract yet. Create one at
`docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`
covering the typed expression AST, public API, schema versioning, release
identity, gate recipe, and the 24-annotation baseline.

## Non-Goals

- Do not duplicate the rtl_frontend contract — the grammars share idioms but
  cover different scopes.
- Do not document features outside `grammars/rtl_const_expr.ebnf`.

## Acceptance Criteria

- A new contract file exists, documenting the AST envelope, expression
  hierarchy (conditional + binop_chain + unary + primary + literal +
  identifier), and the 24-annotation inventory baseline.
- Cross-references to rtl_const_expr_v1.json manifest and (future)
  rtl_const_expr_parser_book paths.

## Task Tree

- ID: `RTL-CE-CONTRACT-BODY`
  Status: `done`
  Goal: `Create the rtl_const_expr integration contract.`
  Children: `RTL-CE-CONTRACT-BODY.1`, `RTL-CE-CONTRACT-BODY.2`,
  `RTL-CE-CONTRACT-BODY.3`

- ID: `RTL-CE-CONTRACT-BODY.1`
  Status: `done`
  Goal: `Create skeleton: Contract Identity, Source Of Truth, Schema Versioning, Release 1.0.1.`
  Acceptance: `Contract file exists with section headings + version numbers + sample inputs (literal_42, binary_addition).`
  Verification: `2026-05-15: PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md created. Identity + Source + Stable surface + Validation gates + Schema versioning + Release 1.0.1 (10 binop_chain levels with consumer-guidance section) populated.`
  Commit: `RTL-CE-CONTRACT-BODY-Slice-1`

- ID: `RTL-CE-CONTRACT-BODY.2`
  Status: `done`
  Goal: `Document AST envelope + expression hierarchy (conditional + binop_chain + unary + primary).`
  Acceptance: `Section enumerates each rule with field list; binop_chain documented as the consumer-facing left-fold contract.`
  Verification: `2026-05-16: "AST Envelope and Expression Hierarchy" contract section added (real 4-field AstDumpPayload + truncation envelope + accuracy note from the VHDL template; rtl_const_expr root; conditional_expr ternary+passthrough; ten-level binop_chain cascade w/ clean [op-envelope,operand] rest + normative left-fold; unary 4 typed + passthrough; primary all-passthrough; literal/identifier leaves + grammar line refs). Independently verified vs generated/rtl_const_expr_return_annotations.json: release 1.0.2 / schema 2 / 26 annotations / 18 distinct rules / 10 binop_chain rules; no dup ## headers. Lockstep DOC-ENVELOPE-0001: rtl_const_expr book comprehensively closed — all 7 affected chapters (ast-envelope/glossary/schema-versioning/walking-the-ast/changelog-index/examples-literal-42/examples-binary-addition) reconciled to the real AstDumpPayload; broad-audit residual 0; NO RTL-CE-Slice-2 regression (schema stays 2, 26/18, clean binop rest, literal/identifier text preserved; <invalid_sequence_access> only as historical framing). rtl_const_expr_parser_book_gate green.`
  Commit: `RTL-CE-CONTRACT-BODY-Slice-2`

- ID: `RTL-CE-CONTRACT-BODY.3`
  Status: `done`
  Goal: `Add literal/identifier shapes, gate recipe, manifest cross-reference, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract cross-references manifest + future book + lib API entry points.`
  Verification: `2026-05-16: Added "### Literal and Identifier Leaf Shapes" (under "## AST Envelope and Expression Hierarchy": literal 2 branches kind based/decimal text:$1 clean post-1.0.2, identifier text:$2 clean post-1.0.2, grammar line refs), "## Companion Documentation" (6-row surface/authority table + precedence; book listed not edited), "### Gate Recipe" (book gate / ast-shape test / on-demand regen — each source-verified: rust/Makefile:745, rust/src/ast_shape_contract.rs:741 unique fn, build-recipe.md flags; Validation/Release Gates referenced by anchor not duplicated), "## Glossary" (14 contract-scoped terms, rtl_const_expr-correct, real 4-field AstDumpPayload). Independently verified: Companion Documentation/Glossary/Gate Recipe/Leaf-Shapes ×1 each; no dup ## headers; numbers 1.0.2/schema 2/26/18/ten-level consistent; zero forbidden reintroductions (no pub root/schema_version-field/pgen_dump_contract_version:u32; pgen_dump_contract_version only truncation/not-a-field ctx); README:240 lists the book (read-only confirm). Discrepancy filed (not hidden): all six README per-parser-book lines use SHELL=/bin/bash vs contracts/build-recipe/Makefile-invocation SHELL=/opt/homebrew/bin/bash — uniform README-wide convention, NOT rtl_const_expr-specific; deferred to a dedicated WORKFLOW slice (cross-family, out of atomic-slice scope), tracked in DEVELOPMENT_NOTES.`
  Commit: `RTL-CE-CONTRACT-BODY-Slice-3`

## Current Frontier

_Empty — tree complete. All leaves `.1`–`.3` `done` (`2026-05-16`)._

## Decisions

- `2026-05-14`: Smaller tree than rtl_frontend (3 leaves vs 4) because the
  expression-focused grammar has less surface to document.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-15` | `RTL-CE-CONTRACT-BODY.1` | manual review of created contract | `pass — Identity through Release 1.0.1 (with consumer-facing binop_chain guidance) populated` |
| `2026-05-16` | `RTL-CE-CONTRACT-BODY.2` | inventory cross-check + dup-header grep + comprehensive book residual audit + no-RTL-CE-Slice-2-regression check + rtl_const_expr_parser_book_gate | `pass — AST Envelope + Expression Hierarchy section (real AstDumpPayload, ten-level binop_chain, 1.0.2/schema 2/26/18/10 verified, no dup ## headers); rtl_const_expr book DOC-ENVELOPE-0001 comprehensively closed (7 chapters, 0 residual, schema 2 unregressed); gate green` |
| `2026-05-16` | `RTL-CE-CONTRACT-BODY.3` | section-count greps (Companion Documentation/Glossary/Gate Recipe/Leaf-Shapes ×1) + dup-`## `-header grep + number-consistency grep + forbidden-reintroduction grep + gate/test/regen source-verification + README book-listing read-only check | `pass — literal/identifier leaf shapes + Companion Documentation (6-row authority table) + Gate Recipe (book gate / ast-shape test / regen, each source-verified to Makefile:745, ast_shape_contract.rs:741, build-recipe.md) + 14-term Glossary added; no dup ## headers; 1.0.2/schema 2/26/18/ten-level consistent; zero forbidden reintroductions (pgen_dump_contract_version only truncation/not-a-field ctx); README:240 lists book. README SHELL-convention divergence (all 6 lines /bin/bash vs /opt/homebrew/bin/bash) flagged + deferred to a WORKFLOW slice (cross-family, out of scope), tracked in DEVELOPMENT_NOTES — reported, not hidden. Tree closed.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-CE-CONTRACT-BODY.1` | `RTL-CE-CONTRACT-BODY-Slice-1` | new contract file: identity + source + stable surface + schema versioning + Release 1.0.1 with binop_chain consumer guidance |
| `RTL-CE-CONTRACT-BODY.2` | `RTL-CE-CONTRACT-BODY-Slice-2` | AST Envelope + Expression Hierarchy section (ten-level binop_chain); rtl_const_expr book DOC-ENVELOPE-0001 comprehensively closed (7 chapters, schema 2 unregressed) |
| `RTL-CE-CONTRACT-BODY.3` | `RTL-CE-CONTRACT-BODY-Slice-3` | literal/identifier leaf shapes + Companion Documentation table + Gate Recipe + 14-term Glossary; closes the tree (7th completed) |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-16`: `.2` done (AST Envelope + Expression Hierarchy section + ten-level binop_chain left-fold; rtl_const_expr book `DOC-ENVELOPE-0001` comprehensively closed in lockstep — 7 chapters, broad-audit 0 residual, no RTL-CE-Slice-2 regression). Frontier advances to `.3` (literal/identifier + gate recipe + cross-refs + links — final leaf).
- `2026-05-16`: `.3` done (literal/identifier leaf shapes + Companion Documentation authority table + source-verified Gate Recipe + 14-term Glossary; independently verified — no dup `## ` headers, numbers consistent, zero forbidden reintroductions, README lists the book). **Tree `RTL-CE-CONTRACT-BODY` complete** — root + all leaves `.1`–`.3` `done`; frontier emptied; promoted to Completed in `docs/TASK_TREE.md` (7th completed tree). README SHELL-convention divergence flagged + deferred to a future WORKFLOW slice (tracked in DEVELOPMENT_NOTES).
