# Public API Surface

The stable downstream contract is the `pgen::embedding_api` host surface, **not** internal generated-parser types. This chapter enumerates the entry points downstream consumers should call.

All entry points return `Outcome` types (`NamedGrammarParseOutcome`, `GrammarAstDumpOutcome`, etc.) that carry an `api_version`, `grammar`, `profile`, `status`, and `diagnostic`/`ast_dump` payload. The `Outcome` types are stable and source-compatible across patch / minor releases.

## Convenience entry points

Since embedding API `2.0.0` (`ENGINE-UNIVERSAL-SERVICES.112`) these live beside the family that
owns them — `pgen::families::systemverilog::embed` — because the neutral API names no language;
each is one line over the generic entry points below.

```rust
use pgen::families::systemverilog::embed::*;

// Parse-only (no AST dump):
pub fn parse_systemverilog_2017(input: &str) -> NamedGrammarParseOutcome;
pub fn parse_systemverilog_2023(input: &str) -> NamedGrammarParseOutcome;

// Parse + AST dump (the version this book documents per-rule):
pub fn parse_systemverilog_2017_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> NamedGrammarAstDumpOutcome;
pub fn parse_systemverilog_2023_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> NamedGrammarAstDumpOutcome;
```

## Generic entry points

For tools that select the grammar family at runtime. `GrammarFamily` and `GrammarProfile` are
RESOLVED values since `2.0.0`: `GrammarFamily::from_str("systemverilog")?` and
`family.profile("sv_2017")?` (every spelling comes from the grammar's own `@profiles` /
`@profile_alias`); the `1.x` enum variants are gone.

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

## String-name overloads

For embedders that select grammar / profile by string name (e.g. plugin systems, cross-language bindings):

```rust
pub fn parse_grammar_profile_named(
    grammar: &str,        // "systemverilog"
    profile: &str,        // "sv_2017" | "sv_2023" | "verilog_2005"
    input: &str,
) -> NamedGrammarParseOutcome;

pub fn parse_grammar_profile_named_with_limits(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
) -> NamedGrammarParseOutcome;
```

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
pub struct NamedGrammarParseOutcome {
    pub api_version: String,
    pub grammar: String,
    pub profile: String,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

pub struct NamedGrammarAstDumpOutcome {
    pub api_version: String,
    pub grammar: String,
    pub profile: String,
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
    pub supports_systemverilog_generated_backend: bool,
    // (other fields)
}
```

Embedders should check this at startup and refuse to operate (or fall back gracefully) if the SV backend is unavailable.

## Stack robustness (embedding API `1.3.1`)

Every SystemVerilog embedding parse runs its generated-parser work on a dedicated
**256 MiB-stack** thread (one short-lived thread per call — concurrent host threads never
serialize on a shared worker). This guarantees the engine's 4096-frame recursion ceiling fires
as a clean `E_PARSE_FAILURE` diagnostic before the OS guard page can abort the host process:
measured pre-fix (`SV-CORPUS-GRAD.8c.3`, 2026-07-22), a ~400-deep parenthesized expression —
≈4 KB of text — was enough to SIGABRT a release embedder at the default 8 MB main stack,
because the ceiling needs ≈8 MB of real stack in release (≈70 MB in debug) to be reachable.
A parser-worker panic likewise surfaces as `E_PARSE_FAILURE`, never a host abort. Host code
needs no change; the guarantee holds on any caller thread, including small worker-pool stacks.

## Standard-library bundle (embedding API `2.1.0`)

Every SystemVerilog parse begins by preloading the IEEE 1800 §G.2 predefined classes (`process`,
`semaphore`, `mailbox`) as `type_name` facts, so `mailbox m;` resolves without user code declaring
the class. Since `2.1.0` (`ENGINE-UNIVERSAL-SERVICES.73`) the bundle is embedded in the binary from
the tracked `parser_libs/sv_{2017,2023}_std/package/std.facts.json`; no run-time path is involved.
`PGEN_PARSER_LIBS_DIR=<dir>` substitutes `<dir>/sv_<profile>_std/package/std.facts.json`, and a
directory named there that lacks the bundle makes the parse fail with a `ParseDiagnostic` whose
message names the knob, the bundle and the expected path — it never falls back to an empty stdlib.
Before `2.1.0` the bundle was resolved from a compile-time absolute path and its absence was silent:
`15.4--mailbox-{blocking,non-blocking}.sv` (both `must_accept`) were rejected on a moved tree.

## Source pointer

The authoritative source for the public API is `rust/src/embedding_api.rs`. The contract document `rust/docs/EMBEDDING_API_CONTRACT.md` describes the stability policy in full.
