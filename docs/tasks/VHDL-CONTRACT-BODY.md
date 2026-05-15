# VHDL-CONTRACT-BODY: Bring VHDL contract to SV parity

## Metadata

- Tree ID: `VHDL-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: vhdl deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Bring `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` to parity
with the SystemVerilog contract: from 55 lines (build/gate boilerplate only)
to a fully-bodied integration contract documenting the typed AST shape that
downstream consumers can rely on.

## Non-Goals

- Do not retrofit per-slice release headings for the SV-style "Release X /
  Contract X Highlights" history. VHDL only has one slice today
  (VHDL-Slice-1); future shape-affecting slices will append release entries
  going forward, not backfill.
- Do not duplicate the future VHDL mdBook. The contract is a downstream
  consumer reference; the book is the user-facing reference.

## Acceptance Criteria

- The contract documents the AST envelope, top-level shape, every rule
  family's `kind` discriminator, the `vhdl_file` root, the design_unit
  dispatch, and the binop_chain expression shape.
- Schema versioning, release identity, sample input, and gate-recipe sections
  exist with concrete content (not placeholders).
- The contract cross-references the upcoming VHDL mdBook (paths defined even
  if the book isn't built yet — book stand-up is `VHDL-MDBOOK`).

## Task Tree

- ID: `VHDL-CONTRACT-BODY`
  Status: `active`
  Goal: `Body out the VHDL integration contract to SV parity.`
  Children: `VHDL-CONTRACT-BODY.1`, `VHDL-CONTRACT-BODY.2`,
  `VHDL-CONTRACT-BODY.3`, `VHDL-CONTRACT-BODY.4`

- ID: `VHDL-CONTRACT-BODY.1`
  Status: `done`
  Goal: `Add Contract Identity, Schema Versioning, and Release 1.0.1 sections.`
  Acceptance: `Contract carries explicit version numbers, annotation count, sample input, and "what changed" pointer to VHDL-Slice-1.`
  Verification: `2026-05-15: Contract Identity (version 1.0.1, schema v1), Schema Versioning table, and Release 1.0.1 Highlights (full 249-annotation rule-by-rule summary) inserted.`
  Commit: `VHDL-CONTRACT-BODY-Slice-1`

- ID: `VHDL-CONTRACT-BODY.2`
  Status: `pending`
  Goal: `Document the AST envelope and design_unit dispatch.`
  Acceptance: `Section enumerates the 10 design_unit kinds with example fields and links to the underlying grammar lines.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-CONTRACT-BODY.3`
  Status: `pending`
  Goal: `Document declarations, types, statements, and expression shapes.`
  Acceptance: `Section enumerates each rule family with kind discriminator and field list; binop_chain documented as the consumer-facing left-fold contract.`
  Verification: `pending`
  Commit: `pending`

- ID: `VHDL-CONTRACT-BODY.4`
  Status: `pending`
  Goal: `Add gate-recipe + cross-references + glossary.`
  Acceptance: `Contract references vhdl_parser_book paths, vhdl_v1.json manifest, and the per-family gate scripts.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `VHDL-CONTRACT-BODY.2` | `pending` | AST envelope + design_unit dispatch deepens the consumer-facing surface. |

## Decisions

- `2026-05-14`: Use SV contract structure as the template; reuse the same
  section headings for cross-grammar consistency.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-15` | `VHDL-CONTRACT-BODY.1` | manual review of inserted sections | `pass — Contract Identity (1.0.1), Schema Versioning (1.0.0/0.1.0 rows), Release 1.0.1 Highlights (249-annotation surface) populated` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VHDL-CONTRACT-BODY.1` | `VHDL-CONTRACT-BODY-Slice-1` | Contract Identity + Schema Versioning + Release 1.0.1 Highlights inserted (240+ lines added to contract) |

## Changelog

- `2026-05-14`: Created task tree.
