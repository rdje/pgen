# PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's `regex` parser family.

This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.

## Current Contract Status
- Current grammar family label:
  - `regex`
- Current stable host profile:
  - `regex_default`
- Current live status:
  - `Done` for the currently tracked grammar contract in `LIVE_ACHIEVEMENT_STATUS.md`
- Practical meaning:
  - PGEN currently treats the tracked `grammars/regex.ebnf` language and its public host surface as closure-grade and fit for downstream parser consumption.

## Source Of Truth
- Grammar source:
  - `grammars/regex.ebnf`
- Tracked generated backend artifact:
  - `generated/regex_parser.rs`
- Tracked frontend JSON artifact:
  - `generated/regex.json`
- Build-time discovery and cfg emission:
  - `rust/build.rs`
  - `PGEN_REGEX_PARSER_PATH`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Family proof/status surfaces:
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Stable Integration Surface
- Grammar family:
  - `regex`
- Stable host profile:
  - `regex_default`
- Stable convenience parse entry points:
  - `parse_regex_default(...)`
  - `parse_regex_default_with_limits(...)`
  - `parse_regex_default_result(...)`
  - `parse_regex_default_with_limits_result(...)`
- Stable convenience AST-dump entry points:
  - `parse_regex_default_ast_dump(...)`
  - `parse_regex_default_ast_dump_with_limits(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable metadata call:
  - `parser_embedding_api_contract()`
  - required startup field:
    - `supports_regex_generated_backend`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`
- Stable integration invariants:
  - `input_ownership_model=borrowed_str`
  - `parse_session_model=stateless_per_call`
  - `zero_copy_input_boundary=true`
  - deterministic by default

## Build / Availability Requirements
- Real downstream use should require the generated regex backend.
- Startup should inspect `parser_embedding_api_contract().supports_regex_generated_backend`.
- If the generated backend is unavailable, the stable failure mode is `E_BACKEND_UNAVAILABLE`.
- The backend is discovered through `rust/build.rs`:
  - default tracked artifact: `generated/regex_parser.rs`
  - override hook: `PGEN_REGEX_PARSER_PATH`

## What A Downstream Project May Rely On
- The host-oriented embedding surface in `pgen::embedding_api` is the downstream contract.
- Bounded input behavior is stable:
  - default `max_input_bytes=1_048_576`
  - overridable via `ParseLimits`
- The AST dump surface is stable as a JSON payload contract:
  - canonicalized JSON string
  - stable truncation envelope if `max_ast_bytes` is exceeded
- Grammar/profile mismatch is stable and deterministic:
  - `regex` only supports `regex_default`
  - mismatched profiles return `E_UNSUPPORTED_PROFILE`

## What This Does Not Promise
- It does not promise that internal generated parser types are stable.
- It does not promise a stable internal Rust AST node schema for direct type-level consumption.
- It does not promise that every regex dialect on earth is covered. The tracked promise is the language defined by `grammars/regex.ebnf`.
- It does not define execution semantics for embedded code blocks such as `(?{...})`; the parser contract is about acceptance, diagnostics, and AST-dump transport, not runtime execution of embedded code.

## Current Grammar Scope Notes
- `grammars/regex.ebnf` is the tracked language contract.
- It currently includes the major families expected by RGX-style consumers:
  - alternation
  - concatenation
  - groups
  - char classes
  - quantifiers
  - anchors
  - lookarounds
  - named groups
  - inline modifiers
  - conditionals
  - embedded code-block syntax
- If RGX needs a wider or intentionally different dialect, that should be treated as an explicit contract-widening task rather than assumed to already be covered by the current `Done` label.

## Downstream Integration Checklist
1. Depend on the stable host module `pgen::embedding_api`.
2. Use profile `regex_default`.
3. Check `parser_embedding_api_contract().supports_regex_generated_backend` at startup or build validation time.
4. Use `parse_regex_default_result(...)` for accept/reject + diagnostics flows.
5. Use `parse_regex_default_ast_dump(...)` only if JSON AST transport is genuinely needed.
6. Treat `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, and `E_INPUT_TOO_LARGE` as first-class expected error modes.
7. Keep a downstream-owned regex acceptance/rejection corpus and run it alongside PGEN’s own gate stack.
8. When reporting a bug, follow `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` so the issue arrives with a reproducible input, structured outcome, AST dump, and trace bundle.
9. Once a real bug is confirmed, expect it to be tracked under the regex family/profile in `PGEN_RELEASED_PARSER_BUG_LEDGER.md` until the fix is released.
10. If RGX or any other downstream consumer also tracks the same bug locally, treat the PGEN ledger as canonical for parser root cause, fix proof, and parser-release state.

## Minimal Rust Example
```rust
use pgen::embedding_api::{
    parser_embedding_api_contract, parse_regex_default_result,
};

let contract = parser_embedding_api_contract();
assert!(contract.supports_regex_generated_backend);

parse_regex_default_result(r"https?://[^\s]+")?;
```

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash regex_parser_integration_contract_gate`
- Family proof/closure:
  - `make -C rust SHELL=/opt/homebrew/bin/bash regex_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash regex_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash regex_combined_telemetry_contract_gate`

## Current Trust Statement
- For the currently tracked regex grammar contract, PGEN’s public position is that downstream projects may rely on this parser through the `embedding_api` host surface.
- If RGX needs a wider dialect or a stable typed AST schema beyond JSON AST-dump transport, that should be treated as new contract work and explicitly versioned here.
