# PNR-SDC: SDC / Tcl-shaped Timing-Constraint Parser Family (EBNF-backed)

## Metadata

- Tree ID: `PNR-SDC`
- Status: `proposed`
- Roadmap lane: `Phase S — RTLSyn Parser Stack (PNR family): SDC timing-constraint reader`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Deliver an EBNF-backed SDC (Synopsys Design Constraints, Tcl-shaped) parser
family per PGEN doctrine (tracked `grammars/sdc.ebnf` → generated parser +
stimuli + proof gates). Reference input note already exists at
`docs/tcl/md/tcl.md` (Tcl syntax shape), but no shipped SDC grammar yet.

## Non-Goals

- Not a constraint/timing solver; parser/front-end only.
- Not a general Tcl interpreter — the SDC command subset RTLSyn needs.
- No handwritten parser as final closure (bootstrap scaffolding only).

## Acceptance Criteria

- Tracked `grammars/sdc.ebnf` + generated parser + stimuli path.
- Roundtrip + coverage + gap proof gates per `README.md` closure doctrine.
- Integration contract + mdBook + AST shape-contract manifest, tri-locked.
- Each completed leaf committed through `COMMIT.md`.

## Task Tree

- ID: `PNR-SDC`
  Status: `proposed`
  Goal: `EBNF-backed SDC/Tcl-shaped constraint parser family to PGEN closure bar.`
  Children: `PNR-SDC.1`

- ID: `PNR-SDC.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): pin the SDC command subset RTLSyn needs (create_clock, set_input/output_delay, set_false_path, etc.) + the Tcl-shaped lexical model (consult docs/tcl/md/tcl.md); define closure bar + first grammar slice. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Scoped command subset + Tcl lexical model + closure bar recorded; first grammar-slice leaf defined.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PNR-SDC.1` | `pending` (proposed) | Scoping precedes grammar work; lane is Not Started. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2`. Not started; activate when prioritized.

## Open Questions

- SDC command subset for the RTLSyn MVP; how much Tcl substitution/eval to model. (resolve in `.1`)

## Blockers

- None (not yet started).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `PNR-SDC.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PNR-SDC.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
