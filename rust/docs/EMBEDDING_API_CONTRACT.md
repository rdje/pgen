# Embedding API Contract (Rust)

## Purpose
Provide a stable, versioned surface for external projects embedding PGEN annotation parsing (for example, HDL frontends and regex tooling) without depending on internal parser/AST implementation details.

## Stable Module
- Rust module: `pgen::embedding_api`
- Contract metadata API:
  - `embedding_api_contract() -> EmbeddingApiContract`
  - `parser_embedding_api_contract() -> ParserEmbeddingApiContract`
- Parse API:
  - Nexsim convenience grammar entry points:
    - `parse_systemverilog_2017(...)`
    - `parse_systemverilog_2023(...)`
    - `parse_vhdl_1076_2019(...)`
    - plus `*_with_limits`, `*_result`, `*_with_limits_result` variants
  - idiomatic Rust `Result` surface:
    - `parse_annotation_result(family, backend, input) -> Result<(), ParseDiagnostic>`
    - `parse_annotation_with_limits_result(family, backend, input, limits) -> Result<(), ParseDiagnostic>`
    - `parse_grammar_profile_result(grammar, profile, input) -> Result<(), ParseDiagnostic>`
    - `parse_grammar_profile_with_limits_result(grammar, profile, input, limits) -> Result<(), ParseDiagnostic>`
  - deterministic structured outcome surface:
  - `parse_annotation(family, backend, input) -> ParseOutcome`
  - `parse_annotation_with_limits(family, backend, input, limits) -> ParseOutcome`
  - `parse_grammar_profile(grammar, profile, input) -> GrammarParseOutcome`
  - `parse_grammar_profile_with_limits(grammar, profile, input, limits) -> GrammarParseOutcome`
  - language-neutral named string surface:
    - `parse_annotation_named(family_name, backend_name, input) -> NamedAnnotationParseOutcome`
    - `parse_annotation_named_with_limits(...) -> NamedAnnotationParseOutcome`
    - `parse_grammar_profile_named(grammar_name, profile_name, input) -> NamedGrammarParseOutcome`
    - `parse_grammar_profile_named_with_limits(...) -> NamedGrammarParseOutcome`
- Parse limits type:
  - `ParseLimits { max_input_bytes }`
  - default via `ParseLimits::default()`
  - default bound constant: `EMBEDDING_API_DEFAULT_MAX_INPUT_BYTES` (`1_048_576`)

## Versioning
- Contract version constant: `EMBEDDING_API_VERSION = "1.0.0"`
- Schema version constant: `EMBEDDING_API_SCHEMA_VERSION = 1`
- Compatibility rules:
  - Major version bump: breaking API or behavioral contract change.
  - Minor/Patch bump: backward-compatible additions/fixes.

## Stable Types
Annotation API:
- `AnnotationFamily`: `return | semantic`
- `ParserBackend`: `bootstrap | generated`
- `ParseStatus`: `success | failure`
- `ParseOutcome`: includes API version, parser family, backend, status, optional diagnostic.
- `ParseDiagnostic`: stable `code` + human-readable `message`.

Grammar parser API:
- `GrammarFamily`: `systemverilog | vhdl`
- `GrammarProfile`: `sv_2017 | sv_2023 | vhdl_1076_2019`
- `InputOwnershipModel`: `borrowed_str`
- `ParseSessionModel`: `stateless_per_call`
- `GrammarParseOutcome`: includes API version, grammar, profile, status, optional diagnostic.
- `NamedAnnotationParseOutcome`: structured result preserving caller-provided family/backend strings.
- `NamedGrammarParseOutcome`: structured result preserving caller-provided grammar/profile strings.
- `ParserEmbeddingApiContract`: stable profile matrix + backend availability flags + integration invariants.
  - includes `profile_matrix` for per-grammar profile lookup.
  - includes zero-copy/session invariants:
    - `input_ownership_model=borrowed_str`
    - `parse_session_model=stateless_per_call`
    - `zero_copy_input_boundary=true`
  - publishes stable parser diagnostic code set via `stable_diagnostic_codes`.

## Diagnostic Code Contract
- `E_BACKEND_UNAVAILABLE`: generated backend requested without `generated_parsers` feature.
- `E_PARSE_FAILURE`: selected backend failed to parse the provided input.
- `E_INPUT_TOO_LARGE`: input exceeds `max_input_bytes` parse limit.
- `E_INVALID_LIMITS`: invalid limit configuration (for example `max_input_bytes == 0`).
- `E_INVALID_ARGUMENT`: unknown family/backend/grammar/profile value in named string APIs.
- `E_UNSUPPORTED_PROFILE`: grammar/profile mismatch (for example `vhdl_1076_2019` used with `systemverilog` grammar entry point).

## Determinism Contract
- `embedding_api_contract().deterministic_by_default` is `true`.
- Parsing uses deterministic execution paths and no random sampling.

## Input-Bound Contract
- `parse_annotation(...)` uses `ParseLimits::default()` and enforces bounded input size.
- Embedders can override the bound per call via `parse_annotation_with_limits(...)`.
- `parse_grammar_profile(...)` uses the same default bounded input behavior.
- Embedders can override the bound per call via `parse_grammar_profile_with_limits(...)`.
- The bound is measured in raw input bytes.

## Grammar Profile Contract (Nexsim-Oriented)
- Stable profile-aware parser entry points exist for host integration:
  - SystemVerilog: `sv_2017`, `sv_2023`
  - VHDL: `vhdl_1076_2019`
- The API enforces grammar/profile compatibility deterministically (`E_UNSUPPORTED_PROFILE` on mismatch).
- Input is accepted as `&str` and consumed without ownership transfer (zero-copy call boundary).
- Session model is intentionally stateless per-call for deterministic embedding behavior:
  - one input payload per call,
  - structured outcome per call,
  - no hidden mutable global parser session state.

## Gate
- Local/CI gate command:
  - `make -C rust embedding_api_gate`
  - `make -C rust nexsim_parser_embedding_contract_gate`
- Gate runs:
  - bootstrap build tests: `cargo test --lib embedding_api`
  - generated build tests: `cargo test --features generated_parsers --lib embedding_api`
  - parser-profile contract subset:
    - `cargo test --lib parser_embedding_`
    - `cargo test --features generated_parsers --lib parser_embedding_`

## Scope Note
- This contract intentionally returns structured parse outcomes instead of exposing internal AST node types. Internal AST representations may evolve as long as contract behavior, versions, and diagnostic codes remain stable.
