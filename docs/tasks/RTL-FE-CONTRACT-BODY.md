# RTL-FE-CONTRACT-BODY: Bring rtl_frontend contract to SV parity

## Metadata

- Tree ID: `RTL-FE-CONTRACT-BODY`
- Status: `done`
- Roadmap lane: rtl_frontend deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

There is no rtl_frontend integration contract yet. Create one at
`docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` covering
the typed AST shape, public API, schema versioning, release identity, and
gate recipe — modeled on the SV contract.

## Non-Goals

- Do not retrofit per-slice release headings for RTL-FE-Slice-1..7. Future
  shape-affecting slices append, current state is the baseline.
- Do not duplicate the future rtl_frontend mdBook.

## Acceptance Criteria

- A new contract file exists, documenting the AST envelope, design_item
  dispatch (4 kinds), module/package structure, expression core
  (binop_chain), and the 156-annotation inventory baseline.
- Cross-references to rtl_frontend_v1.json manifest and (future)
  rtl_frontend_parser_book paths.

## Task Tree

- ID: `RTL-FE-CONTRACT-BODY`
  Status: `done`
  Goal: `Create the rtl_frontend integration contract.`
  Children: `RTL-FE-CONTRACT-BODY.1`, `RTL-FE-CONTRACT-BODY.2`,
  `RTL-FE-CONTRACT-BODY.3`, `RTL-FE-CONTRACT-BODY.4`
  Result: `All 4 children done. The rtl_frontend integration contract is at SV parity: Identity/Source/Schema/Release-1.0.1 (.1), AST Envelope + design_item/module_item/generate_item dispatch (.2), Declarations/Types/Ports/Statements/Expressions + ten-level binop_chain left-fold (.3), Companion Documentation + Gate Recipe + Glossary (.4). The rtl_frontend book DOC-ENVELOPE-0001 was comprehensively closed in lockstep in .2 (all 7 chapters, broad-audit-verified 0 residual). Tree complete 2026-05-16; 6th completed tree.`

- ID: `RTL-FE-CONTRACT-BODY.1`
  Status: `done`
  Goal: `Create the contract skeleton with Contract Identity + Source Of Truth + Schema Versioning + Release 1.0.1 sections.`
  Acceptance: `Contract file exists; section headings + version numbers + sample input populated.`
  Verification: `2026-05-15: docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md created (188 lines); Identity, Source Of Truth, Stable Surface, Build/Validation gates, Schema Versioning + 1.0.0/0.1.0 rows, and full Release 1.0.1 Highlights rule-by-rule across all 7 RTL-FE slices populated.`
  Commit: `RTL-FE-CONTRACT-BODY-Slice-1`

- ID: `RTL-FE-CONTRACT-BODY.2`
  Status: `done`
  Goal: `Document AST envelope + design_item / module_item / generate_item dispatch.`
  Acceptance: `Section enumerates 4/10/11 kinds with field lists and links to grammar lines.`
  Verification: `2026-05-16: "AST Envelope and Dispatch" contract section added (real 4-field AstDumpPayload + truncation envelope + accuracy note, copied from the proven VHDL contract; rtl_frontend_file root; design_item=4 / module_item=10 / generate_item=11 kinds with field shapes + verified grammars/rtl_frontend.ebnf line refs). Verified 156/74 + 4/10/11 vs generated/rtl_frontend_return_annotations.json; no dup ## headers. Lockstep DOC-ENVELOPE-0001: the rtl_frontend book is now COMPREHENSIVELY 0-residual — fixed all 7 affected chapters (ast-envelope/glossary/schema-versioning/walking-the-ast/changelog-index via subagent; json-carrier + examples-empty-module added after a broad re-audit caught the worked-example dump.root/dump.schema_version vector the narrow grep missed). rtl_frontend_parser_book_gate green.`
  Commit: `RTL-FE-CONTRACT-BODY-Slice-2`

- ID: `RTL-FE-CONTRACT-BODY.3`
  Status: `done`
  Goal: `Document declarations, types, ports, statements, expressions (binop_chain).`
  Acceptance: `Section enumerates each rule family; binop_chain documented as left-fold contract.`
  Verification: `2026-05-16: "Declarations, Types, Ports, Statements, and Expressions" section added (7 rule families w/ kinds+fields + grammar line refs; the TEN-level binop_chain table logical_or..multiplicative + normative left-fold contract, no sign field, ternary/unary passthrough). Independently verified vs generated/rtl_frontend_return_annotations.json: 156/74, exactly 10 binop_chain rules; no duplicate ## headers; AstDumpPayload 4-field model untouched/clean (not reintroduced). Consistent with the verified rtl_frontend rules-top-level.md; no contract-vs-book discrepancy. Contract-only (rtl_frontend book DOC-ENVELOPE-0001 already fully closed in Slice-2).`
  Commit: `RTL-FE-CONTRACT-BODY-Slice-3`

- ID: `RTL-FE-CONTRACT-BODY.4`
  Status: `done`
  Goal: `Add gate-recipe, manifest cross-reference, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract cross-references the manifest + future book + lib API entry points.`
  Verification: `2026-05-16: "Companion Documentation" (6-row surface/authority table + precedence order), "### Gate Recipe" (4 commands each verified vs build-recipe.md / Makefile:739 / ast_shape_contract.rs:763), and a 12-term "Glossary" appended. grep -c Companion Documentation==1, Glossary==1, no duplicate ## headers, real 4-field AstDumpPayload only. README.md already lists the rtl_frontend book in its Per-Parser Integration Reference Books section (line 239, confirmed read-only; not modified). Minor noted-only: README uses SHELL=/bin/bash vs the contract's /opt/homebrew/bin/bash (both valid; contract internally consistent).`
  Commit: `RTL-FE-CONTRACT-BODY-Slice-4`

## Current Frontier

_None — the `RTL-FE-CONTRACT-BODY` tree is complete. All leaves `.1`–`.4` are `done`._

## Decisions

- `2026-05-14`: Model after `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
  structure.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-15` | `RTL-FE-CONTRACT-BODY.1` | manual review of created contract | `pass — 188-line contract created with full rule-by-rule surface for 156 annotations` |
| `2026-05-16` | `RTL-FE-CONTRACT-BODY.2` | inventory cross-check + dup-header grep + comprehensive book residual audit + rtl_frontend_parser_book_gate | `pass — AST Envelope + Dispatch section (real AstDumpPayload, 4/10/11 kinds, grammar line refs, 156/74 verified, no dup ## headers); rtl_frontend book DOC-ENVELOPE-0001 comprehensively closed (7 chapters, 0 residual incl. worked-example vector); gate green` |
| `2026-05-16` | `RTL-FE-CONTRACT-BODY.3` | inventory cross-check + dup-header grep + AstDumpPayload-clean check | `pass — Declarations/Types/Ports/Statements/Expressions section (7 families + ten-level binop_chain left-fold); 156/74 + 10 binop rules verified; no dup ## headers; AstDumpPayload 4-field model untouched; consistent with verified book` |
| `2026-05-16` | `RTL-FE-CONTRACT-BODY.4` | command/path verification + dup-header grep + AstDumpPayload-clean + README read-only check | `pass — Companion Documentation + Gate Recipe + Glossary appended (commands verified vs build-recipe.md/Makefile:739/ast_shape_contract.rs:763); grep -c==1 each; no dup ## headers; real 4-field AstDumpPayload; README already lists the book. Tree complete.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-CONTRACT-BODY.1` | `RTL-FE-CONTRACT-BODY-Slice-1` | new contract file: identity + source + stable surface + schema versioning + Release 1.0.1 full rule-by-rule |
| `RTL-FE-CONTRACT-BODY.2` | `RTL-FE-CONTRACT-BODY-Slice-2` | AST Envelope + Dispatch section (4/10/11); rtl_frontend book DOC-ENVELOPE-0001 fully closed (7 chapters, 0 residual) |
| `RTL-FE-CONTRACT-BODY.3` | `RTL-FE-CONTRACT-BODY-Slice-3` | Declarations/Types/Ports/Statements/Expressions section (ten-level binop_chain left-fold); contract-only |
| `RTL-FE-CONTRACT-BODY.4` | `RTL-FE-CONTRACT-BODY-Slice-4` | Companion Documentation + Gate Recipe + Glossary; **tree complete** |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-16`: `.2` done (AST Envelope + Dispatch section; rtl_frontend book `DOC-ENVELOPE-0001` comprehensively closed — 7 chapters, 0 residual, broad audit caught the worked-example `dump.root`/`dump.schema_version` vector the earlier narrow grep missed). Frontier advances to `.3` (declarations/types/ports/statements/expressions), then `.4`.
- `2026-05-16`: `.3` done (Declarations/Types/Ports/Statements/Expressions section + ten-level binop_chain left-fold contract; contract-only — book DOC-ENVELOPE-0001 already closed). Frontier advances to `.4` (gate-recipe + manifest cross-ref + README/LIVE links — final leaf).
- `2026-05-16`: `.4` done (Companion Documentation + Gate Recipe + Glossary; README already lists the book). All children `done`; **`RTL-FE-CONTRACT-BODY` tree closed** and moved to Completed Task Trees in `docs/TASK_TREE.md` — 6th completed tree.
