# Public API Surface

The stable downstream contract is the `pgen::embedding_api` host surface, **not** internal generated-parser types. This chapter enumerates the entry points downstream VHDL consumers should call.

All entry points return `Outcome` types (`GrammarParseOutcome`, `GrammarAstDumpOutcome`, etc.) that carry an `api_version`, `grammar`, `profile`, `status`, and `diagnostic`/`ast_dump` payload. The `Outcome` types are stable and source-compatible across patch / minor releases.

## Convenience entry points

```rust
// Parse-only (no AST dump):
pub fn parse_vhdl_1076_2019(input: &str) -> GrammarParseOutcome;

pub fn parse_vhdl_1076_2019_with_limits(
    input: &str,
    limits: &ParseLimits,
) -> GrammarParseOutcome;

pub fn parse_vhdl_1076_2019_result(input: &str) -> Result<(), ParseDiagnostic>;

pub fn parse_vhdl_1076_2019_with_limits_result(
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic>;

// Parse + AST dump (the version this book documents per-rule):
pub fn parse_vhdl_1076_2019_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome;

pub fn parse_vhdl_1076_2019_ast_dump_with_limits(
    input: &str,
    options: &AstDumpOptions,
    limits: &ParseLimits,
) -> GrammarAstDumpOutcome;
```

`vhdl_1076_2019` corresponds to IEEE 1076-2019. It is currently the only stable VHDL profile.

## Generic entry points

For tools that select the grammar family at runtime:

```rust
pub fn parse_grammar_profile(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
) -> GrammarParseOutcome;

pub fn parse_grammar_profile_result(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
) -> Result<(), ParseDiagnostic>;

pub fn parse_grammar_profile_ast_dump(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome;
```

Use `GrammarFamily::Vhdl` + `GrammarProfile::Vhdl1076_2019`.

## String-name overloads

For embedders that select grammar / profile by string name (e.g. plugin systems, cross-language bindings):

```rust
pub fn parse_grammar_profile_named(
    grammar: &str,        // "vhdl"
    profile: &str,        // "vhdl_1076_2019" | "1076-2019" | "ieee1076-2019"
    input: &str,
) -> NamedGrammarParseOutcome;

pub fn parse_grammar_profile_named_with_limits(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
) -> NamedGrammarParseOutcome;
```

Recognized aliases for the VHDL 1076-2019 profile name: `"vhdl_1076_2019"`, `"1076-2019"`, `"ieee1076-2019"`, `"ieee_1076_2019"`.

## AstDumpOptions

```rust
pub struct AstDumpOptions {
    pub pretty: bool,                 // pretty-print the JSON
    pub max_ast_bytes: Option<usize>, // truncation cap (bytes); None = no cap
}
```

## ParseLimits

```rust
pub struct ParseLimits {
    pub max_input_bytes: Option<usize>,  // input-size cap; None = no cap
    // (other fields may be added in minor releases)
}

impl Default for ParseLimits {
    fn default() -> Self;
}
```

## Outcome types

```rust
pub struct GrammarParseOutcome {
    pub api_version: String,
    pub grammar: GrammarFamily,
    pub profile: GrammarProfile,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

pub struct GrammarAstDumpOutcome {
    pub api_version: String,
    pub grammar: GrammarFamily,
    pub profile: GrammarProfile,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
    pub ast_dump: Option<AstDumpPayload>,
}

pub enum ParseStatus { Success, Failure }

pub struct ParseDiagnostic {
    pub code: String,        // e.g. "E_PARSE_FAILURE"
    pub message: String,
    // (other fields)
}
```

## Stable diagnostics

These diagnostic codes are stable across patch / minor releases:

- `E_BACKEND_UNAVAILABLE` — the generated parser backend is not present in this build.
- `E_PARSE_FAILURE` — the input failed to parse.
- `E_INPUT_TOO_LARGE` — the input exceeds the configured `max_input_bytes` limit.
- `E_INVALID_LIMITS` — the supplied `ParseLimits` are not valid.
- `E_INVALID_ARGUMENT` — the supplied grammar / profile / input is not valid.
- `E_UNSUPPORTED_PROFILE` — the supplied profile is not supported by the selected grammar family.

## Backend availability check

```rust
pub fn parser_embedding_api_contract() -> ParserEmbeddingApiContract;

pub struct ParserEmbeddingApiContract {
    pub supports_vhdl_generated_backend: bool,
    // (other fields)
}
```

Embedders should check this at startup and refuse to operate (or fall back gracefully) if the VHDL backend is unavailable.

## Source pointer

The authoritative source for the public API is `rust/src/embedding_api.rs`. The contract document `rust/docs/EMBEDDING_API_CONTRACT.md` describes the stability policy in full.
