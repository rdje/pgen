# VHDL-CONTRACT-BODY: Bring VHDL contract to SV parity

## Metadata

- Tree ID: `VHDL-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: vhdl deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
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
  Status: `done`
  Goal: `Document the AST envelope and design_unit dispatch.`
  Acceptance: `Section enumerates the 10 design_unit kinds with example fields and links to the underlying grammar lines.`
  Verification: `2026-05-16: "AST Envelope and design_unit Dispatch" section added (real 4-field AstDumpPayload + truncation envelope + accuracy note, vhdl_file root, 10-branch design_unit dispatch with per-kind body shapes + verified grammars/vhdl.ebnf line refs). Independently verified vs generated/vhdl_return_annotations.json: 249 annotations / 110 distinct rules / 10 design_unit branches. Also fixed a duplicate "## Source Of Truth" header (grep -c == 1). Surfaced the systemic fabricated-AstDumpPayload doc defect: fixed docs/vhdl_parser_book/src/ast-envelope.md in lockstep (VHDL contract+book consistent; vhdl_parser_book_gate green); other 4 books tracked as DOC-ENVELOPE-0001 (DEVELOPMENT_NOTES).`
  Commit: `VHDL-CONTRACT-BODY-Slice-2`

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
| 1 | `VHDL-CONTRACT-BODY.3` | `pending` | Declarations / types / statements / expression (binop_chain) shapes — deepens the per-rule-family consumer surface. |
| 2 | `VHDL-CONTRACT-BODY.4` | `pending` | Gate-recipe + cross-references + glossary close the contract. |

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
| `2026-05-16` | `VHDL-CONTRACT-BODY.2` | inventory cross-check + dedup grep + vhdl_parser_book_gate | `pass — AST Envelope + 10-branch design_unit dispatch section (real AstDumpPayload, grammar line refs, 249/110/10 verified); duplicate Source Of Truth header merged (grep -c==1); VHDL book ast-envelope.md reconciled to the real struct, gate green; systemic 4-book defect tracked DOC-ENVELOPE-0001` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VHDL-CONTRACT-BODY.1` | `VHDL-CONTRACT-BODY-Slice-1` | Contract Identity + Schema Versioning + Release 1.0.1 Highlights inserted (240+ lines added to contract) |
| `VHDL-CONTRACT-BODY.2` | `VHDL-CONTRACT-BODY-Slice-2` | AST Envelope + design_unit dispatch section; dup-header fix; VHDL book ast-envelope.md reconciled; DOC-ENVELOPE-0001 tracked |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-16`: `.2` done (AST Envelope + design_unit dispatch; dup-header fix; VHDL book ast-envelope.md reconciled to the real `AstDumpPayload`; systemic 4-book doc defect tracked as `DOC-ENVELOPE-0001`). Frontier advances to `.3` (declarations/types/statements/expression shapes), then `.4`.
