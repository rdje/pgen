# PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `vhdl` parser family.

## Source Of Truth
- Grammar source:
  - `grammars/vhdl.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_VHDL_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`

## Stable Integration Surface
- Grammar family:
  - `vhdl`
- Stable host profile:
  - `vhdl_1076_2019`
- Stable convenience entry points:
  - `parse_vhdl_1076_2019(...)`
  - `parse_vhdl_1076_2019_ast_dump(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should inspect `parser_embedding_api_contract().supports_vhdl_generated_backend` during startup or build validation.
- The generated backend is resolved by `rust/build.rs`, not by importing internal generated parser modules directly.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate`
- Family closure/proof:
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_combined_telemetry_contract_gate`

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `vhdl` is still an `In Progress` family in the live tracker, so downstream integrators should treat the embedding surface as real but still pay attention to the current live blocker list in `LIVE_ACHIEVEMENT_STATUS.md`.
- When reporting downstream bugs, follow `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
