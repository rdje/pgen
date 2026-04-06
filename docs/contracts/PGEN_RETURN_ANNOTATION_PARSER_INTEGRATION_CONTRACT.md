# docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `return_annotation` parser family.

## Source Of Truth
- Main grammar source:
  - `grammars/return_annotation.ebnf`
- Bootstrap-safe grammar source:
  - `grammars/builtin_return_annotation.ebnf`
- Tracked generated artifacts:
  - `generated/return_annotation.json`
  - `generated/return_annotation_parser.rs`
- Public host API:
  - `rust/src/embedding_api.rs`
- Normative semantic/contract doc:
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`

## Stable Integration Surface
- Annotation family:
  - `return`
- Stable host entry points:
  - `parse_annotation(...)`
  - `parse_annotation_result(...)`
  - `parse_annotation_named(...)`
- Stable family selector:
  - `AnnotationFamily::Return`
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
- Generated backend is also part of the published contract when `generated_parsers` support is present.
- Downstream consumers should use the embedding API surface rather than linking directly to generated parser modules.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash annotation_contract_gate`
- `make -C rust SHELL=/bin/bash return_runtime_semantics_gate`
- `make -C rust SHELL=/bin/bash return_annotation_support_gate`

## Scope / Non-Goals
- The downstream contract is parser acceptance, diagnostics, and the annotation family/backend selection surface.
- Internal typed AST conversion logic in the Rust AST pipeline is not itself the generic downstream parser contract.
- `return_annotation` is currently a `Done` family for the tracked claim, but that claim is still defined by the repo’s current grammar and proof stack, not by informal future expectations.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
