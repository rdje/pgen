# PNR-AUX-READERS: Auxiliary PNR Readers (gate-level netlist / config / optional SDF)

## Metadata

- Tree ID: `PNR-AUX-READERS`
- Status: `proposed`
- Roadmap lane: `Phase S — RTLSyn Parser Stack (PNR family): later auxiliary readers`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Deliver the EBNF-backed auxiliary readers the RTLSyn/PNR flow needs after the
core families: gate-level (structural Verilog) netlist reader, config reader,
and an optional SDF (Standard Delay Format) reader — each per PGEN doctrine
(tracked EBNF → generated parser + stimuli + proof gates).

## Non-Goals

- Not analysis/back-end engines; parsers/front-ends only.
- Note: a structural Verilog netlist may reuse/derive from the existing Verilog
  2005 extracted grammar — `.1` decides reuse vs. dedicated subset.
- No handwritten parser as final closure.

## Acceptance Criteria

- Each shipped reader: tracked `grammars/<reader>.ebnf` + generated parser +
  stimuli + roundtrip/coverage/gap gates.
- Integration contracts + mdBooks + AST shape-contract manifests, tri-locked.
- Each completed leaf committed through `COMMIT.md`.

## Task Tree

- ID: `PNR-AUX-READERS`
  Status: `proposed`
  Goal: `EBNF-backed gate-level netlist / config / optional SDF readers to PGEN closure bar.`
  Children: `PNR-AUX-READERS.1`

- ID: `PNR-AUX-READERS.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): enumerate which aux readers the RTLSyn MVP actually requires + their priority; decide gate-level-netlist reuse of grammars/verilog_2005_lrm_extracted.ebnf vs a dedicated structural subset; define closure bar + first reader. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Reader list + priority + reuse decision + closure bar recorded; first reader leaf defined.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PNR-AUX-READERS.1` | `pending` (proposed) | Scoping precedes grammar work; lane is Not Started (lowest PNR priority). |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2`. Not started; activate after PNR-LIBERTY/PNR-SDC if prioritized.

## Open Questions

- Is SDF in or out of the RTLSyn MVP? Reuse Verilog-2005 grammar for the netlist reader? (resolve in `.1`)

## Blockers

- None (not yet started).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `PNR-AUX-READERS.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PNR-AUX-READERS.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
