# SVPP-CONTRACT-BODY: Bring sv_preprocessor contract to SV parity

## Metadata

- Tree ID: `SVPP-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: sv_preprocessor deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Body out `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
(currently a thin scaffolding) to SV-equivalent depth: typed AST shape,
public API, schema versioning, release identity, gate recipe, and the
64-annotation baseline.

## Non-Goals

- Do not duplicate the future sv_preprocessor mdBook.
- Do not document SV LRM preprocessor semantics outside the current grammar
  scope.

## Acceptance Criteria

- The contract documents the AST envelope, pp_item dispatch (10 kinds),
  directive shapes (define/undef/include/timescale/default_nettype/celldefine),
  conditional-compilation tree, macro_body fragment, and the 64-annotation
  baseline.
- Cross-references to `systemverilog_preprocessor_v1.json` and (future) book.

## Task Tree

- ID: `SVPP-CONTRACT-BODY`
  Status: `active`
  Goal: `Body out the sv_preprocessor integration contract.`
  Children: `SVPP-CONTRACT-BODY.1`, `SVPP-CONTRACT-BODY.2`,
  `SVPP-CONTRACT-BODY.3`, `SVPP-CONTRACT-BODY.4`

- ID: `SVPP-CONTRACT-BODY.1`
  Status: `done`
  Goal: `Audit current contract content; populate Contract Identity + Schema Versioning + Release 1.0.1.`
  Acceptance: `Contract carries explicit version numbers, annotation count, sample input (single_define).`
  Verification: `2026-05-15: Identity (1.0.1), Schema Versioning (1.0.0/0.1.0 rows), Release 1.0.1 Highlights (full 64-annotation surface including pp_item dispatch + 7 directives + conditional tree + macro fragment dispatch) inserted at the top of the existing scope-limited contract.`
  Commit: `SVPP-CONTRACT-BODY-Slice-1`

- ID: `SVPP-CONTRACT-BODY.2`
  Status: `pending`
  Goal: `Document AST envelope + pp_item dispatch + per-directive shapes.`
  Acceptance: `Section enumerates 10 pp_item kinds + 7 directive shapes with field lists.`
  Verification: `pending`
  Commit: `pending`

- ID: `SVPP-CONTRACT-BODY.3`
  Status: `pending`
  Goal: `Document conditional-compilation tree + macro_body fragment (9 kinds) + macro_default_atom (8 kinds).`
  Acceptance: `Section enumerates all fragment kinds with consumer guidance.`
  Verification: `pending`
  Commit: `pending`

- ID: `SVPP-CONTRACT-BODY.4`
  Status: `pending`
  Goal: `Add gate recipe + manifest cross-reference + README/LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract references manifest + future book + lib API entry points.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SVPP-CONTRACT-BODY.2` | `pending` | Deeper pp_item / directive shape documentation builds on the now-existing identity section. |

## Decisions

- `2026-05-14`: Model after the SV contract structure for cross-grammar
  consistency.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-15` | `SVPP-CONTRACT-BODY.1` | manual review of inserted sections | `pass — Identity through Release 1.0.1 surface populated at the top of the existing scope-limited contract` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SVPP-CONTRACT-BODY.1` | `SVPP-CONTRACT-BODY-Slice-1` | Contract Identity + Schema Versioning + Release 1.0.1 Highlights (full 64-annotation surface) inserted at the top |

## Changelog

- `2026-05-14`: Created task tree.
