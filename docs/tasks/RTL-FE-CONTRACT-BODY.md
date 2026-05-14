# RTL-FE-CONTRACT-BODY: Bring rtl_frontend contract to SV parity

## Metadata

- Tree ID: `RTL-FE-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: rtl_frontend deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
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
  Status: `active`
  Goal: `Create the rtl_frontend integration contract.`
  Children: `RTL-FE-CONTRACT-BODY.1`, `RTL-FE-CONTRACT-BODY.2`,
  `RTL-FE-CONTRACT-BODY.3`, `RTL-FE-CONTRACT-BODY.4`

- ID: `RTL-FE-CONTRACT-BODY.1`
  Status: `pending`
  Goal: `Create the contract skeleton with Contract Identity + Source Of Truth + Schema Versioning + Release 1.0.1 sections.`
  Acceptance: `Contract file exists; section headings + version numbers + sample input populated.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CONTRACT-BODY.2`
  Status: `pending`
  Goal: `Document AST envelope + design_item / module_item / generate_item dispatch.`
  Acceptance: `Section enumerates 4/10/11 kinds with field lists and links to grammar lines.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CONTRACT-BODY.3`
  Status: `pending`
  Goal: `Document declarations, types, ports, statements, expressions (binop_chain).`
  Acceptance: `Section enumerates each rule family; binop_chain documented as left-fold contract.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CONTRACT-BODY.4`
  Status: `pending`
  Goal: `Add gate-recipe, manifest cross-reference, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract cross-references the manifest + future book + lib API entry points.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-FE-CONTRACT-BODY.1` | `pending` | Skeleton must exist before content. |

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
| `2026-05-14` | `RTL-FE-CONTRACT-BODY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-CONTRACT-BODY.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree.
