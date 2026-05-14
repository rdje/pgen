# RTL-CE-CONTRACT-BODY: Bring rtl_const_expr contract to SV parity

## Metadata

- Tree ID: `RTL-CE-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: rtl_const_expr deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
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
  Status: `active`
  Goal: `Create the rtl_const_expr integration contract.`
  Children: `RTL-CE-CONTRACT-BODY.1`, `RTL-CE-CONTRACT-BODY.2`,
  `RTL-CE-CONTRACT-BODY.3`

- ID: `RTL-CE-CONTRACT-BODY.1`
  Status: `pending`
  Goal: `Create skeleton: Contract Identity, Source Of Truth, Schema Versioning, Release 1.0.1.`
  Acceptance: `Contract file exists with section headings + version numbers + sample inputs (literal_42, binary_addition).`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-CE-CONTRACT-BODY.2`
  Status: `pending`
  Goal: `Document AST envelope + expression hierarchy (conditional + binop_chain + unary + primary).`
  Acceptance: `Section enumerates each rule with field list; binop_chain documented as the consumer-facing left-fold contract.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-CE-CONTRACT-BODY.3`
  Status: `pending`
  Goal: `Add literal/identifier shapes, gate recipe, manifest cross-reference, README + LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract cross-references manifest + future book + lib API entry points.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-CE-CONTRACT-BODY.1` | `pending` | Skeleton must exist before content. |

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
| `2026-05-14` | `RTL-CE-CONTRACT-BODY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-CE-CONTRACT-BODY.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree.
