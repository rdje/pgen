# PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `semantic_annotation` parser family.

## Source Of Truth
- Main grammar source:
  - `grammars/semantic_annotation.ebnf`
- Bootstrap-safe grammar source:
  - `grammars/builtin_semantic_annotation.ebnf`
- Tracked generated artifacts:
  - `generated/semantic_annotation.json`
  - `generated/semantic_annotation_parser.rs`
- Public host API:
  - `rust/src/embedding_api.rs`
- Normative semantic/contract docs:
  - `PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`

## Stable Integration Surface
- Annotation family:
  - `semantic`
- Stable host entry points:
  - `parse_annotation(...)`
  - `parse_annotation_result(...)`
  - `parse_annotation_named(...)`
- Stable family selector:
  - `AnnotationFamily::Semantic`
- Stable backend selectors:
  - `ParserBackend::Bootstrap`
  - `ParserBackend::Generated`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`

## Build / Availability Requirements
- Bootstrap backend is part of the published contract.
- Generated backend is part of the published contract when the generated annotation parser is available.
- Downstream consumers should use the embedding API surface rather than directly depending on internal generated parser types.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash annotation_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_usage_gate`
- `make -C rust SHELL=/bin/bash semantic_runtime_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_full_contract_gate`

## Scope / Non-Goals
- This contract covers parser-family selection, acceptance/rejection, diagnostics, and the current bootstrap/generated host surface.
- Semantic-runtime meaning, steering leverage, and aggregate proof obligations are governed by `PGEN_ANNOTATION_NORMATIVE_SPEC.md` and `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, not by this file alone.
- `semantic_annotation` does not currently have a separate top-level live-status row; track its maturity through the annotation proof spine and the docs above.
- When reporting downstream bugs, follow `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
