# Live Achievement Status

Last updated: 2026-03-14

## Purpose
Provide a precise, always-current progress surface for the project using exactly four status levels:
- `Done`
- `Mostly Done`
- `In Progress`
- `Not Started`

This file is the authoritative live tracking view for "where we are now".

## Status Rules
- `Done`: exit criteria for the tracked area are implemented, validated, and no material roadmap gap remains for that area.
- `Mostly Done`: the core implementation is landed and validated, but bounded follow-up work is still required before closure.
- `In Progress`: meaningful implementation has started, but core capabilities or validation are still missing.
- `Not Started`: no meaningful implementation has landed yet.

## Update Policy
- Review and update this file before every commit when a task changes actual project closure, remaining scope, or the next most important gap.
- When any live-status row changes, log that change here before commit and explicitly surface the changed snapshot in the user-facing completion message for that task.
- When a task does not change live status, say that status is unchanged rather than implying drift.
- Use only the four statuses above.
- Keep "Evidence" concrete and "Left To Close" explicit.

## Live Snapshot

### Major Roadmap Phases

Phase completion tracks whether a roadmap phase delivered its stated contract. It does not automatically mean every parser family touched by those phases is equally mature; parser-family maturity is normalized separately below.

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| Phases A-R | Done | Roadmap phases `A` through `R` currently show only completed checklist items. | Nothing material inside the currently tracked phase checklists. |
| Phase S overall: RTLSyn parser stack minimum viable coverage | In Progress | `rtl_const_expr` and `rtl_frontend` are active, executable crates with ongoing implementation and passing tests, but the phase now has an explicit EBNF-only closure rule. | Replace bootstrap handwritten parser coverage with tracked EBNF-backed generated parser paths and start the still-missing companion parser crates. |

### Parser Family Status

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `systemverilog` main parser (`Phase P` Nexsim scope) | Done | Phase `P` is closed for the tracked Nexsim-facing contract: dual-profile `sv_2017` / `sv_2023` grammar support, required strict `sv_stimuli_quality_gate`, semantic contract suites, tracked `100%` parse-full floor under the phase contract, realistic-corpus proof, embedding API profile contract, and aggregate SOTA enforcement are all in place. | Nothing material inside the currently tracked Phase `P` scope. If the project later wants a broader "full IEEE 1800 beyond current Nexsim contract" target, that should be introduced as a new roadmap phase instead of silently reopening Phase `P`. |
| `systemverilog_preprocessor` frontend (`Phase Q`) | Done | Phase `Q` is closed: dedicated preprocessor EBNF, deterministic preprocessing engine, source maps/event logs, parser-backed quality gate, curated/template differential gates, and required aggregate policy enforcement are all tracked. | Nothing material inside the currently tracked Phase `Q` scope. |
| `vhdl` parser family | In Progress | Executable `grammars/vhdl.ebnf`, readiness gate coverage, parser-backed `vhdl_stimuli_quality_gate`, realistic-corpus proof, strict-promotion telemetry, and required aggregate policy are landed, but the roadmap only delivered a Phase `O` readiness kickoff rather than a dedicated VHDL production-closure phase comparable to SystemVerilog Phase `P`. | Define and execute a dedicated VHDL full-closure phase if the target is a professional parser at SystemVerilog-grade rigor: broader syntax/semantic closure, stronger realistic-corpus proof, differential/reference hardening, and explicit embedder-facing closure criteria. |
| `regex` parser family | In Progress | `regex.ebnf` participates in the Rust-native EBNF frontend readiness and dual-run parity work, and the raw-AST parity audit is closed in favor of the Rust frontend, but there is no dedicated regex parser-quality/realistic-corpus/embedding closure program comparable to the SystemVerilog flow. | Add a dedicated regex closure phase if the target is a professional parser rather than grammar-readiness only: parser-quality gates, corpus-backed proof, and explicit embedding/diagnostic closure criteria. |

### Phase S Detailed Breakdown

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `rtl_const_expr` baseline evaluator | In Progress | Standalone crate exists and covers an executable handwritten baseline, but the parser side is not yet backed by tracked EBNF/PGEN generation. | Land a tracked constant-expression EBNF plus generated parser path, then close the remaining evaluator coverage gaps. |
| `rtl_frontend` synthesizable subset baseline | In Progress | Current subset covers a large executable handwritten baseline, but final closure now requires a tracked RTL-subset EBNF plus a PGEN-generated parser path. | Land the RTL-subset EBNF/generated parser path and continue closing the remaining mixed-expression/procedural/dataflow gaps. |
| Liberty parser crate | Not Started | Roadmap item still open; no crate/worktree implementation is tracked yet. | Add the crate and land the minimum timing/Boolean/area extraction subset. |
| SDC parser crate | Not Started | Roadmap item still open; no crate/worktree implementation is tracked yet. | Add the crate and land the planned minimum constraint subset. |
| Later auxiliary readers (`gate-level` netlist reader, config reader, optional SDF) | Not Started | Still listed as later/non-day-1 items in Phase S only. | Start only after the core parser-stack MVP is materially closer to closure. |

### Immediate Next Gap

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| Phase S EBNF-backed closure | In Progress | The project direction is now explicit: every Phase S parser must be EBNF-backed through PGEN, with handwritten parsers counting only as bootstrap scaffolding. | Add tracked EBNFs and generated parser paths for the current handwritten baselines, starting with `rtl_frontend` and `rtl_const_expr`. |
