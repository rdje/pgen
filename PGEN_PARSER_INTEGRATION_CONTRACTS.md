# PGEN_PARSER_INTEGRATION_CONTRACTS.md

## Purpose
Define the version-controlled downstream integration-contract surface for parser families published by PGEN.

These documents exist so a downstream project can be pointed at one stable `.md` file for the parser family it wants to consume, instead of reverse-engineering readiness from chat history, scattered gate names, or internal Rust modules.

Issue-reporting protocol for any integrated parser:
- `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `PGEN_RELEASED_PARSER_BUG_LEDGER.md`

## Rule
- Every current and future parser family that PGEN publishes for downstream consumption must have a tracked integration-contract document.
- If a parser family does not yet have a stable downstream host API, its family document must say that explicitly instead of implying readiness.
- A parser family should not claim painless downstream integration unless its family document, its stable API surface, and at least one executable gate agree.

## Current Family Documents

| Family | Integration Contract | Primary Stable Surface | Notes |
|---|---|---|---|
| `systemverilog` | `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Nexsim-facing host profile contract. |
| `systemverilog_preprocessor` | `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | `rust/src/sv_preprocessor.rs` runtime stage | Explicitly documents that a generic public embedding API is not published yet. |
| `vhdl` | `PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Nexsim-facing host profile contract. |
| `regex` | `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Downstream-ready regex contract for RGX and other regex consumers. |
| `return_annotation` | `PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` annotation parse API | Family-specific contract layered on top of the aggregate annotation proof spine. |
| `semantic_annotation` | `PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` annotation parse API | Family-specific contract layered on top of the aggregate annotation proof spine. |

## Required Shape For Each Family Document
- `Purpose`
- `Contract Identity`
- `Source Of Truth`
- `Stable Integration Surface`
- `Build / Availability Requirements`
- `Validation / Release Gates`
- `Scope / Non-Goals`

Recommended additions when a family is actively consumed by another project:
- contract version
- parser release version
- last updated stamp
- a downstream-specific checklist
- representative pass/fail examples
- explicit startup checks
- a short “what this does not promise” section
- a pointer to `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

## Operational Rule
- When a downstream project asks “how do I integrate parser X?”, point it first to the family document listed here.
- When a downstream project reports a released-parser bug, require the repro bundle from `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log the accepted report in `PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the matching parser family/profile.
- GitHub is not required for that loop; local git-tracked records in PGEN plus zero-or-more downstream consumer repos are sufficient.
- When the family document and the code disagree, fix the upstream source of truth first, then bring the family document back into parity.
- When a new parser family becomes a real downstream target, add its family document here in the same change that exposes or stabilizes the host surface.
