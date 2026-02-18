# Embedding API Contract (Rust)

## Purpose
Provide a stable, versioned surface for external projects embedding PGEN annotation parsing (for example, HDL frontends and regex tooling) without depending on internal parser/AST implementation details.

## Stable Module
- Rust module: `pgen::embedding_api`
- Contract metadata API:
  - `embedding_api_contract() -> EmbeddingApiContract`
- Parse API:
  - `parse_annotation(family, backend, input) -> ParseOutcome`

## Versioning
- Contract version constant: `EMBEDDING_API_VERSION = "1.0.0"`
- Schema version constant: `EMBEDDING_API_SCHEMA_VERSION = 1`
- Compatibility rules:
  - Major version bump: breaking API or behavioral contract change.
  - Minor/Patch bump: backward-compatible additions/fixes.

## Stable Types
- `AnnotationFamily`: `return | semantic`
- `ParserBackend`: `bootstrap | generated`
- `ParseStatus`: `success | failure`
- `ParseOutcome`: includes API version, parser family, backend, status, optional diagnostic.
- `ParseDiagnostic`: stable `code` + human-readable `message`.

## Diagnostic Code Contract
- `E_BACKEND_UNAVAILABLE`: generated backend requested without `generated_parsers` feature.
- `E_PARSE_FAILURE`: selected backend failed to parse the provided input.

## Determinism Contract
- `embedding_api_contract().deterministic_by_default` is `true`.
- Parsing uses deterministic execution paths and no random sampling.

## Gate
- Local/CI gate command:
  - `make -C rust embedding_api_gate`
- Gate runs:
  - bootstrap build tests: `cargo test --lib embedding_api`
  - generated build tests: `cargo test --features generated_parsers --lib embedding_api`

## Scope Note
- This contract intentionally returns structured parse outcomes instead of exposing internal AST node types. Internal AST representations may evolve as long as contract behavior, versions, and diagnostic codes remain stable.
