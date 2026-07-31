# PNR-LIBERTY: Liberty (.lib) Parser Family (EBNF-backed)

## Metadata

- Tree ID: `PNR-LIBERTY`
- Status: `proposed`
- Roadmap lane: `Phase S — RTLSyn Parser Stack (PNR family): Liberty timing-library reader`
- Created: `2026-05-31`
- Owner: repo-local workflow

## Goal

Deliver an EBNF-backed Liberty (`.lib`) parser family per PGEN doctrine
(tracked `grammars/liberty.ebnf` → `generated/liberty_parser.rs` + stimuli +
roundtrip/coverage proof gates), to the same closure bar as the shipped
families. Liberty is the technology-library reader the RTLSyn/PNR flow needs.

## Non-Goals

- Not a timing-analysis engine; this is the parser/front-end only.
- No handwritten parser as final closure (bootstrap scaffolding only, per
  Phase S closure rule).

## Acceptance Criteria

- Tracked `grammars/liberty.ebnf` + generated parser + stimuli path.
- Roundtrip + coverage + gap proof gates, per `README.md` closure doctrine.
- Per-parser integration contract (`docs/contracts/`) + mdBook + AST
  shape-contract manifest, tri-locked with the codebase.
- Each completed leaf committed through `COMMIT.md`.

## Task Tree

- ID: `PNR-LIBERTY`
  Status: `proposed`
  Goal: `EBNF-backed Liberty parser family to PGEN closure bar.`
  Children: `PNR-LIBERTY.1`

- ID: `PNR-LIBERTY.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): pin the Liberty subset RTLSyn actually needs (library/cell/pin/timing-group structure), survey reference grammar sources, and define the closure bar + first grammar slice. Tools-first; no code until this leaf's scope is accepted and a code leaf owns it.`
  Acceptance: `A scoped subset + closure-bar definition recorded in this tree; first grammar-slice leaf defined.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PNR-LIBERTY.1` | `pending` (proposed) | Scoping must precede any grammar work; lane is Not Started. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` so the Phase S Liberty lane is task-tree-tracked per doctrine. Not started; activate when prioritized.

## Open Questions

- Which Liberty version/subset is the RTLSyn MVP? (resolve in `.1`)

## Blockers

- None (not yet started; lower priority than active SV-EXH-PROOF + governance work).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `PNR-LIBERTY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PNR-LIBERTY.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
