# Embedding API Contract (Rust)

## Purpose
Provide a stable, versioned surface for external projects embedding PGEN annotation parsing and selected grammar parsing (for example, HDL frontends and regex tooling) without depending on internal parser/AST implementation details.

Root-level parser-family handoff docs (`PGEN_*_PARSER_INTEGRATION_CONTRACT.md`) layer family-specific integration guidance on top of this general API contract.

## Stable Module
- Rust module: `pgen::embedding_api`
- Contract metadata API:
  - `embedding_api_contract() -> EmbeddingApiContract`
  - `parser_embedding_api_contract() -> ParserEmbeddingApiContract`
- Parse API:
  - host-oriented convenience grammar entry points:
    - `parse_systemverilog_2017(...)`
    - `parse_systemverilog_2023(...)`
    - `parse_vhdl_1076_2019(...)`
    - `parse_regex_default(...)`
    - plus `*_with_limits`, `*_result`, `*_with_limits_result` variants
  - host-oriented convenience grammar AST-dump entry points:
    - `parse_systemverilog_2017_ast_dump(...)`
    - `parse_systemverilog_2023_ast_dump(...)`
    - `parse_vhdl_1076_2019_ast_dump(...)`
    - `parse_regex_default_ast_dump(...)`
    - plus `*_ast_dump_with_limits` variants
  - idiomatic Rust `Result` surface:
    - `parse_annotation_result(family, backend, input) -> Result<(), ParseDiagnostic>`
    - `parse_annotation_with_limits_result(family, backend, input, limits) -> Result<(), ParseDiagnostic>`
    - `parse_grammar_profile_result(grammar, profile, input) -> Result<(), ParseDiagnostic>`
    - `parse_grammar_profile_with_limits_result(grammar, profile, input, limits) -> Result<(), ParseDiagnostic>`
    - `parse_grammar_profile_ast_dump_result(grammar, profile, input, options) -> Result<AstDumpPayload, ParseDiagnostic>`
    - `parse_grammar_profile_ast_dump_with_limits_result(grammar, profile, input, limits, options) -> Result<AstDumpPayload, ParseDiagnostic>`
  - deterministic structured outcome surface:
  - `parse_annotation(family, backend, input) -> ParseOutcome`
  - `parse_annotation_with_limits(family, backend, input, limits) -> ParseOutcome`
  - `parse_grammar_profile(grammar, profile, input) -> GrammarParseOutcome`
  - `parse_grammar_profile_with_limits(grammar, profile, input, limits) -> GrammarParseOutcome`
  - `parse_grammar_profile_ast_dump(grammar, profile, input, options) -> GrammarAstDumpOutcome`
  - `parse_grammar_profile_ast_dump_with_limits(grammar, profile, input, limits, options) -> GrammarAstDumpOutcome`
  - language-neutral named string surface:
    - `parse_annotation_named(family_name, backend_name, input) -> NamedAnnotationParseOutcome`
    - `parse_annotation_named_with_limits(...) -> NamedAnnotationParseOutcome`
    - `parse_grammar_profile_named(grammar_name, profile_name, input) -> NamedGrammarParseOutcome`
    - `parse_grammar_profile_named_with_limits(...) -> NamedGrammarParseOutcome`
    - `parse_grammar_profile_ast_dump_named(grammar_name, profile_name, input, options) -> NamedGrammarAstDumpOutcome`
    - `parse_grammar_profile_ast_dump_named_with_limits(...) -> NamedGrammarAstDumpOutcome`
- Parse limits type:
  - `ParseLimits { max_input_bytes }`
  - default via `ParseLimits::default()`
  - default bound constant: `EMBEDDING_API_DEFAULT_MAX_INPUT_BYTES` (`1_048_576`)
- AST dump options type:
  - `AstDumpOptions { pretty, max_ast_bytes }`
  - default via `AstDumpOptions::default()`

## Versioning
- Contract version constant: `EMBEDDING_API_VERSION = "1.3.2"`
  - `1.3.2` (2026-08-21, `ENGINE-UNIVERSAL-SERVICES.43`): backward-compatible fix completing
    `1.3.1` — crossing the 4096-frame recursion ceiling now returns its clean `E_PARSE_FAILURE`
    diagnostic in **bounded time**. `1.3.1` guaranteed the host's stack; it did not bound the
    parse, and crossing the ceiling had come to mean an unbounded search rather than a
    rejection (measured on the shipped SV parser: 315 nested parens accepted in 0.16 s, 320
    returned no result in 30 s). Root cause and fix are engine-level and grammar-agnostic —
    see the Stack-Robustness Contract section below. No API surface, struct-shape or schema
    change (schema stays `2`); no accept-set change — a sub-ceiling control is accepted at
    exactly the same depth as before.
  - `1.3.1` (2026-07-22, `SV-CORPUS-GRAD.8c.3`): backward-compatible stack-robustness fix —
    SystemVerilog and VHDL grammar-family parses now run on a dedicated 256 MiB-stack thread,
    so over-deep recursion surfaces as a clean `E_PARSE_FAILURE` diagnostic (the engine's
    4096-frame recursion ceiling) instead of a host-process stack-overflow SIGABRT (measured:
    a ~400-deep parenthesized SV expression — ≈4 KB of text — hard-aborted a release embedder
    at the default 8 MB main stack). See the Stack-Robustness Contract section below. The regex
    path is unchanged (it keeps its own RGX-0085 dedicated worker + nesting pre-check). Schema
    version unchanged at `2`; struct shapes unchanged.
  - `1.3.0` (2026-07-02, `VERILOG-2005-PROFILE.2`): backward-compatible addition of the
    `verilog_2005` SystemVerilog profile to `GrammarProfile` / `supported_profiles` / `profile_matrix`
    (schema version unchanged at `2`; struct shape unchanged). The profile is registered and
    selectable; its strict IEEE-1364-2005 subset enforcement on the SV grammar is being hardened
    incrementally (tracked by the `VERILOG-2005-PROFILE` task tree).
- Schema version constant: `EMBEDDING_API_SCHEMA_VERSION = 2`
- Compatibility rules:
  - Major version bump: breaking API or behavioral contract change.
  - Minor/Patch bump: backward-compatible additions/fixes.

## Stable Types
Annotation API:
- `AnnotationFamily`: `return | semantic`
- `ParserBackend`: `bootstrap | generated`
- `ParseStatus`: `success | failure`
- `ParseOutcome`: includes API version, parser family, backend, status, optional diagnostic.
- `ParseDiagnostic`: stable `code` + human-readable `message` + optional `location`.
  - `location.byte_offset`
  - `location.line`
  - `location.column`

Grammar parser API:
- `GrammarFamily`: `systemverilog | vhdl | regex`
- `GrammarProfile`: `sv_2017 | sv_2023 | verilog_2005 | vhdl_1076_2019 | regex_default`
  - `verilog_2005` (aliases `1364-2005` / `ieee1364-2005` / `ieee_1364_2005`) is a strict IEEE
    1364-2005 (Verilog) parsing profile on the SystemVerilog grammar: it rides the `sv_2017` baseline
    of shared core constructs and rejects SV-only constructs. Subset enforcement is being completed
    incrementally (see the `VERILOG-2005-PROFILE` task tree); today it gates `class_declaration`.
  - `PROFILE-ALIAS.2` (2026-07-07, SURFACE-NEUTRAL maintenance note): the SystemVerilog alias
    spellings this enum's `FromStr` accepts (`2017` / `ieee1800-2017` / `ieee_1800_2017`, the
    `2023` group, the `1364-2005` group) are now DECLARED BY THE GRAMMAR (`@profile_alias` in
    `grammars/systemverilog.ebnf`) and carried by the generated parser itself (a sorted
    `GRAMMAR_PROFILE_ALIASES` constant + case-insensitive resolution inside `set_grammar_profile`).
    The typed `FromStr` spellings remain part of THIS contract and are drift-gated: a features-on
    test (`grammar_profile_from_str_matches_the_artifact_declared_alias_map`) asserts the enum
    resolves every grammar-declared spelling to the declared canonical profile, so the two
    surfaces cannot silently diverge. No accepted spelling changed; no version bump.
- `InputOwnershipModel`: `borrowed_str`
- `ParseSessionModel`: `stateless_per_call`
- `GrammarParseOutcome`: includes API version, grammar, profile, status, optional diagnostic.
- `GrammarAstDumpOutcome`: includes API version, grammar, profile, status, optional diagnostic, optional `ast_dump`.
- `NamedAnnotationParseOutcome`: structured result preserving caller-provided family/backend strings.
- `NamedGrammarParseOutcome`: structured result preserving caller-provided grammar/profile strings.
- `NamedGrammarAstDumpOutcome`: structured AST-dump result preserving caller-provided grammar/profile strings.
- `AstDumpPayload`: canonical JSON payload string + truncation metadata:
  - `dump_json`
  - `truncated`
  - `full_bytes`
  - `emitted_bytes`
- `ParserEmbeddingApiContract`: stable profile matrix + backend availability flags + integration invariants.
  - includes `profile_matrix` for per-grammar profile lookup.
  - includes stable machine-localizable diagnostic field names via `stable_diagnostic_location_fields`.
  - includes zero-copy/session invariants:
    - `input_ownership_model=borrowed_str`
    - `parse_session_model=stateless_per_call`
      - this is a host-API/session guarantee, not a claim that the generated parser implementation is internally stateless
      - generated parser instances remain stateful during a call and may carry mutable per-instance state such as parse cursor, memoization, recursion guards, and semantic-runtime state
      - callers should still treat every public parse entrypoint invocation as an independent parse session
    - `zero_copy_input_boundary=true`
  - publishes stable parser diagnostic code set via `stable_diagnostic_codes`.
  - publishes regex downstream contract metadata:
    - `regex_integration_contract_version`
    - `regex_parser_release_version`
    - `regex_ast_dump_schema_version`
    - `regex_generated_backend_required_feature`
    - `regex_generated_backend_required_artifact`
    - `regex_generated_backend_env_override`
    - `regex_frontend_json_artifact`
    - `regex_frontend_json_role`

## Diagnostic Code Contract
- `E_BACKEND_UNAVAILABLE`: generated backend requested without `generated_parsers` feature.
- `E_PARSE_FAILURE`: selected backend failed to parse the provided input.
- `E_INPUT_TOO_LARGE`: input exceeds `max_input_bytes` parse limit.
- `E_INVALID_LIMITS`: invalid limit configuration (for example `max_input_bytes == 0`).
- `E_INVALID_ARGUMENT`: unknown family/backend/grammar/profile value in named string APIs.
- `E_UNSUPPORTED_PROFILE`: grammar/profile mismatch (for example `vhdl_1076_2019` used with `systemverilog` grammar entry point).
- When the selected backend can localize a parse failure precisely, `ParseDiagnostic.location` is part of the stable diagnostic payload.
- Generated regex parse failures are expected to populate that location object.

## Determinism Contract
- `embedding_api_contract().deterministic_by_default` is `true`.
- Parsing uses deterministic execution paths and no random sampling.
- AST dump payloads are recursively canonicalized by JSON key order before encoding.

## Input-Bound Contract
- `parse_annotation(...)` uses `ParseLimits::default()` and enforces bounded input size.
- Embedders can override the bound per call via `parse_annotation_with_limits(...)`.
- `parse_grammar_profile(...)` uses the same default bounded input behavior.
- Embedders can override the bound per call via `parse_grammar_profile_with_limits(...)`.
- The bound is measured in raw input bytes.

## Stack-Robustness Contract (`SV-CORPUS-GRAD.8c.3`)
- The generated parsers carry a 4096-frame mutual-recursion ceiling; a ceiling only protects
  the host if the thread it runs on has enough real stack for 4096 frames (measured for
  SystemVerilog: ≈2 KB/frame release, ≈17 KB/frame debug ⇒ ≈8 MB / ≈70 MB of stack needed —
  more than the 8 MB default main stack, far more than typical host worker threads).
- Therefore every SystemVerilog and VHDL grammar-family parse (`parse_grammar_profile*`,
  the convenience/`_result`/`_ast_dump` wrappers) runs its generated-parser work on a
  dedicated thread with a **256 MiB stack** (virtual reservation, lazily committed;
  spawn-per-call, so concurrent host threads never serialize on a shared worker).
- Guarantee: pathologically deep input (e.g. a ~400-level-nested expression) yields a clean
  `E_PARSE_FAILURE` diagnostic — the host process is **never** aborted by a parser stack
  overflow, in either build mode. A worker panic likewise maps to `E_PARSE_FAILURE`
  (`generated <family> parser worker panicked while processing input`).
- ⭐ **And that diagnostic arrives in BOUNDED TIME** (`ENGINE-UNIVERSAL-SERVICES.43`, `1.3.2`).
  A bound that converts into a search is not a bound: for a host parsing machine-generated or
  untrusted input, a few KB of legal-looking text that never returns is a denial of service, and
  it is *harder* to diagnose than the abort `1.3.1` replaced — a crash fails fast and loud, a hang
  holds the caller's thread. The engine's ceiling now behaves as the resource bound it is:
  - a cycle-guard verdict (`Infinite` / `LeftRecursive`) names a blocking FRAME and its failures
    stay out of the packrat cache, because replaying one from a different stack would refuse a
    parse the guard would have allowed;
  - the whole-stack DEPTH ceiling names no frame, and its failures ARE cached, under a **depth
    stamp**: re-entering the same rule at the same position from a deeper stack prunes everything
    the first attempt pruned and possibly more, so a failure at depth `D` holds at every depth
    `>= D`, and that is a condition the cache entry can carry.
  Locked by `parser_embedding_systemverilog_over_ceiling_nesting_fails_in_bounded_time` and its
  VHDL twin (over the ceiling: clean diagnostic within a wall-clock budget) plus
  `parser_embedding_systemverilog_under_ceiling_nesting_still_parses` (under it: still accepted,
  so bounded time cannot be bought by lowering the ceiling).
- The regex family keeps its own, earlier-established defense (the RGX-0085 dedicated
  64 MiB worker + `REGEX_MAX_NESTING_DEPTH` pre-parse ceiling) — unchanged by this contract
  addition, preserving the regex performance floor.

## AST Dump Contract
- AST dump options are provided by `AstDumpOptions`.
  - `pretty=false` emits compact canonical JSON.
  - `pretty=true` emits canonical pretty JSON.
  - `max_ast_bytes=None` is unbounded.
  - `max_ast_bytes=Some(N)` enforces bounded AST dump output (`N >= 1`).
- If encoded AST payload exceeds `max_ast_bytes`, AST payload is replaced by deterministic truncation diagnostics JSON envelope:
  - `kind = "pgen_ast_dump_truncation"`
  - `dump_kind = "parser_return_ast"`
  - `max_bytes`
  - `full_bytes`
  - `reason`
- If `max_ast_bytes` is too small to fit truncation diagnostics envelope itself, API returns `E_INVALID_LIMITS`.
- Family-specific AST schema promises are defined in the matching family integration contract.
  - For regex specifically:
    - schema version is exposed as `parser_embedding_api_contract().regex_ast_dump_schema_version`
    - the stable schema definition lives in `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`

## Grammar Profile Contract (Host-Oriented)
- Stable profile-aware parser entry points exist for host integration:
  - SystemVerilog: `sv_2017`, `sv_2023`
  - VHDL: `vhdl_1076_2019`
  - regex: `regex_default`
- The API enforces grammar/profile compatibility deterministically (`E_UNSUPPORTED_PROFILE` on mismatch).
- Published downstream regex integrations should also use `parser_embedding_api_contract()` to discover:
  - generated-backend availability
  - regex parser release version
  - regex integration contract version
  - regex generated-backend requirements
- Downstream bug reports should copy those exact version values from `parser_embedding_api_contract()` rather than inferring them from stale docs or memory.
- Input is accepted as `&str` and consumed without ownership transfer (zero-copy call boundary).
- Session model is intentionally stateless per-call for deterministic embedding behavior:
  - one input payload per call,
  - structured outcome per call,
  - no hidden mutable global parser session state.

## Gate
- Local/CI gate command:
  - `make -C rust embedding_api_gate`
  - `make -C rust nexsim_parser_embedding_contract_gate`
  - `make -C rust regex_parser_integration_contract_gate`
- Gate runs:
  - bootstrap build tests: `cargo test --lib embedding_api`
  - generated build tests: `cargo test --features generated_parsers --lib embedding_api`
  - Nexsim parser-profile contract subset:
    - `cargo test --lib parser_embedding_`
    - `cargo test --features generated_parsers --lib parser_embedding_`
- Scope note:
  - `embedding_api_gate` now covers the public regex parser/profile surface too.
  - `nexsim_parser_embedding_contract_gate` remains the stricter SV/VHDL host-profile contract slice.
  - `regex_parser_integration_contract_gate` is the downstream-facing regex consumer slice layered on top of the generic embedding API contract.

## Scope Note
- This contract intentionally returns structured parse outcomes instead of exposing internal AST node types. Internal AST representations may evolve as long as contract behavior, versions, and diagnostic codes remain stable.
