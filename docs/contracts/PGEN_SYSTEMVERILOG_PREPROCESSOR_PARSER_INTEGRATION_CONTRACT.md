# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `systemverilog_preprocessor` frontend/parsing stage.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog_preprocessor.ebnf`
- Runtime execution stage:
  - `rust/src/sv_preprocessor.rs`
- Generated-parser build discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
- Current operational guide:
  - `PGEN_USER_GUIDE.md`
- Live status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`

## Stable Integration Surface
- Current downstream-facing contract is narrower than the main SystemVerilog/VHDL/regex host surface.
- The repository does expose generated-parser registry coverage for `systemverilog_preprocessor`, but it does not currently publish a dedicated general-purpose embedding API profile for it in `pgen::embedding_api`.
- The practical stable surface today is:
  - the Rust preprocessor execution/runtime module in `rust/src/sv_preprocessor.rs`
  - the executable quality and differential gates documented in `PGEN_USER_GUIDE.md`

## Build / Availability Requirements
- Do not treat internal parser-registry exposure as equivalent to a published general-purpose downstream host contract.
- If a downstream project needs a generic public embedding API for `systemverilog_preprocessor`, that should be treated as new product-surface work, not assumed from current internal registry availability.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate`

## Scope / Non-Goals
- This document is intentionally explicit that `systemverilog_preprocessor` does not yet have the same published host-embedding shape as `systemverilog`, `vhdl`, or `regex`.
- Downstream consumers should not couple themselves to internal generated parser modules as if they were already a stable public API.
- If a downstream integrator still reports a reproducible preprocessor/runtime bug, use `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log accepted released-parser issues in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
