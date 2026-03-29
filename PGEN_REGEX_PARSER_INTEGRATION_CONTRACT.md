# PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's `regex` parser family.

This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.

## Contract Identity
- Contract version:
  - `1.1.0`
- Parser release version:
  - `1.1.0`
- Embedding API contract baseline:
  - `1.2.0`
- Regex AST-dump schema version:
  - `1`
- Last updated:
  - `2026-03-29`
- Current grammar family label:
  - `regex`
- Current stable host profile:
  - `regex_default`
- Current live status:
  - `Done` for the currently tracked grammar contract in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- PGEN currently treats the published regex flavor, when consumed through the stable `pgen::embedding_api` host surface, as closure-grade and fit for downstream parser consumption.
- That statement applies to the published regex parser contract documented here and in the regex-flavor section of `PGEN_USER_GUIDE.md`.
- It does not automatically cover every regex dialect or every future contract widening.

## Release 1.1.0 Highlights
- `1.1.0` is the first regex handoff release that rolls forward the recent PCRE2 hardening slice instead of only documenting the older closed family row.
- New published syntax coverage in `1.1.0` includes:
  - negated POSIX classes such as `[[:^alnum:]]`
  - braced named backreferences such as `\k{name}`
  - bare-name and signed numeric conditional references
  - left-open counted quantifiers such as `{,4}` and comma-only counted form `{,}`
- The generated regex host path in `1.1.0` also enforces a small compile-style validation contract after parse success, so obvious compile-invalid forms no longer slip through as successful parses.
- The release is additionally backed by the maintained PCRE2 compile-oracle lane documented in `PGEN_USER_GUIDE.md`.

## Supporting Documents
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Published regex flavor and operator-facing guidance:
  - `PGEN_USER_GUIDE.md`
- Shared issue-reporting protocol:
  - `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- Canonical released-parser bug ledger:
  - `PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- Family proof/status surfaces:
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Stable Integration Surface
- Grammar family:
  - `regex`
- Stable host profile:
  - `regex_default`
- Stable metadata calls:
  - `embedding_api_contract()`
  - `parser_embedding_api_contract()`
- Stable regex convenience parse entry points:
  - `parse_regex_default(...)`
  - `parse_regex_default_with_limits(...)`
  - `parse_regex_default_result(...)`
  - `parse_regex_default_with_limits_result(...)`
- Stable regex convenience AST-dump entry points:
  - `parse_regex_default_ast_dump(...)`
  - `parse_regex_default_ast_dump_with_limits(...)`
- Stable generic grammar parse entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_with_limits(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_with_limits_result(...)`
- Stable generic grammar AST-dump entry points:
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_ast_dump_with_limits(...)`
  - `parse_grammar_profile_ast_dump_result(...)`
  - `parse_grammar_profile_ast_dump_with_limits_result(...)`
- Stable named-string entry points for bindings and adapters:
  - `parse_grammar_profile_named(...)`
  - `parse_grammar_profile_named_with_limits(...)`
  - `parse_grammar_profile_named_with_limits_result(...)`
  - `parse_grammar_profile_ast_dump_named(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits_result(...)`
- Stable regex contract metadata available through `parser_embedding_api_contract()`:
  - `supports_regex_generated_backend`
  - `regex_integration_contract_version`
  - `regex_parser_release_version`
  - `regex_ast_dump_schema_version`
- Stable integration invariants:
  - `input_ownership_model=borrowed_str`
  - `parse_session_model=stateless_per_call`
  - `zero_copy_input_boundary=true`
  - deterministic by default

## Build / Availability Requirements
- Real downstream use should require the generated regex backend.
- Startup or build validation should inspect:
  - `parser_embedding_api_contract().supports_regex_generated_backend`
- If building directly from a PGEN checkout, enable the generated parser surface rather than relying on bootstrap-only builds.
- If the generated backend is unavailable, the stable failure mode is:
  - `E_BACKEND_UNAVAILABLE`

## Stable Diagnostics Contract
- Stable diagnostic codes:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`
- Parse diagnostics now expose a stable optional machine-localizable location object:
  - `location.byte_offset`
  - `location.line`
  - `location.column`
- The location object is emitted when the selected parser backend can localize the parse failure precisely.
- Regex parse failures through the generated regex backend are expected to populate this location object.

## Stable AST-Dump Schema Contract
- Regex AST-dump JSON is transport-stable and schema-stable at schema version `1`.
- Schema version `1` stabilizes the recursive node envelope and variant encoding:
  - top-level node object fields:
    - `rule_name`
    - `span`
    - `content`
  - `span` fields:
    - `start`
    - `end`
  - `content` is an externally tagged JSON object with exactly one active variant:
    - `Terminal`
    - `TransformedTerminal`
    - `Sequence`
    - `Alternative`
    - `Quantified`
- Representative shape:

```json
{
  "rule_name": "regex",
  "span": {
    "start": 0,
    "end": 3
  },
  "content": {
    "Sequence": [
      {
        "rule_name": "atom",
        "span": {
          "start": 0,
          "end": 1
        },
        "content": {
          "Terminal": "a"
        }
      }
    ]
  }
}
```

- This schema contract is about JSON shape, field names, and variant encoding.
- Downstream consumers that interpret specific `rule_name` values should pin to a parser release version and rerun their own AST compatibility suite on upgrade.
- This document does not promise stable internal Rust AST node types.

## Current External Hardening Baseline
- The maintained PCRE2 compile-oracle lane is now part of the downstream trust story for the regex release.
- Current tracked baseline:
  - `cases_executed=2195`
  - `expected_parse_ok_total=1613`
  - `expected_parse_fail_total=582`
  - `parse_expectation_match_total=1668`
  - `parse_expectation_mismatch_total=527`
  - `false_accept_total=325`
  - `false_reject_total=202`
- This does not reopen the closed `regex` family row by itself, but it is the main maintained future hardening lane for downstream trust widening.

## Published Regex Flavor Summary
- The currently published regex parser accepts:
  - empty regex
  - raw regex bodies, not host-language delimiter wrappers
  - alternation and concatenation
  - capturing, noncapturing, named, and atomic groups
  - lookahead and lookbehind assertions
  - greedy, lazy, and possessive quantifiers
  - counted quantifier forms such as `{3}`, `{2,}`, `{2,4}`, `{,4}`, and `{,}`
  - char classes, negated char classes, ranges, and POSIX classes
  - negated POSIX classes such as `[[:^alnum:]]`
  - anchors including `^`, `$`, `\A`, `\Z`, `\z`, `\b`, `\B`, and `\G`
  - backreferences including `\1`, `\k<name>`, `\k'name'`, and `\k{name}`
  - inline modifiers and scoped modifiers
  - conditional regex forms whose condition may be:
    - a lookaround assertion
    - a bare name
    - an explicit name reference
    - digits
    - signed digits
    - a recursion condition
  - embedded code-block syntax such as `(?{...})` and language-tagged variants
- Embedded code-block parser-layer contract:
  - plain `(?{...})` is preserved as opaque generic payload
  - `lua`, `js`, and `javascript` payloads are preserved as opaque source-body payloads
  - parser-layer structural handling currently guarantees:
    - balanced braces
    - single-quoted strings
    - double-quoted strings
    - escaped characters
- Generated-host compile-contract safeguards:
  - the generated regex host path additionally rejects several obvious compile-invalid forms even if the raw grammar shape parsed successfully
  - currently enforced rejections include:
    - unsupported `\i`
    - counted quantifier minimum/maximum inversions such as `{5,4}`
    - counted quantifier bounds above `65535`
    - forbidden character-class escapes such as `[\B]`, `[\R]`, and `[\X]`
    - descending character-class ranges such as `[z-a]`
    - quantified anchors such as `^+`
    - variable-length lookbehind such as `(?<=a+)b`
- The current detailed flavor description and measured operational baseline live in `PGEN_USER_GUIDE.md`.
- The current parser contract does not promise:
  - slash-delimited host literal parsing such as `/pattern/flags` as a dedicated wrapper syntax
  - arbitrary valid Lua or JavaScript source acceptance beyond the structural forms listed above
  - JavaScript comment/template-literal shielding or Lua long-bracket shielding as part of the current published parser contract
  - `native` or `wasm` tagged code blocks as part of the current published parser contract
  - runtime execution semantics for embedded code blocks
  - semantic equivalence with every regex engine on earth
  - a stable typed Rust AST API beyond the JSON schema described above

## Downstream Integration Checklist
1. Depend on the stable host module `pgen::embedding_api`.
2. Use profile `regex_default`.
3. Check `parser_embedding_api_contract().supports_regex_generated_backend` at startup or build validation time.
4. Record both:
   - `parser_embedding_api_contract().regex_parser_release_version`
   - `parser_embedding_api_contract().regex_integration_contract_version`
   - and copy those exact values into bug reports instead of inferring them from memory or stale docs
5. Use `parse_regex_default_result(...)` for accept/reject + diagnostics flows.
6. Use `parse_regex_default_ast_dump(...)` only if JSON AST transport is genuinely needed.
7. Treat `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, and `E_INPUT_TOO_LARGE` as first-class expected error modes.
8. Keep a downstream-owned regex acceptance/rejection corpus and run it alongside PGEN's own gate stack.
9. When reporting a bug, follow the quick path below and the full protocol in `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
10. Once a real bug is confirmed, expect it to be tracked under the regex family/profile in `PGEN_RELEASED_PARSER_BUG_LEDGER.md` until the fix is released.

## Issue Reporting Quick Path
- Every actionable regex parser bug report should include:
  - PGEN commit ID or released crate version
  - regex parser release version
  - regex integration contract version
  - parser family/profile:
    - `regex`
    - `regex_default`
  - exact failing input file
  - expected vs actual behavior
  - pretty-printed `parser_embedding_api_contract()` JSON
  - pretty-printed parse outcome JSON
  - pretty-printed AST dump JSON when the parse succeeds but the structure is wrong
- If the downstream project can run a PGEN checkout, also capture parser trace artifacts:

```bash
export PGEN_TRACE_VERBOSITY=debug

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse regex repro_input.txt \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty regex repro_input.txt pgen_ast_dump.json \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log
```

- If the downstream project only has the Rust embedding API, capture:
  - `parser_embedding_api_contract()`
  - `parse_regex_default(...)` or `parse_regex_default_result(...)`
  - `parse_regex_default_ast_dump(...)` when AST structure is relevant
- For version fields, copy directly from:
  - `parser_embedding_api_contract().regex_parser_release_version`
  - `parser_embedding_api_contract().regex_integration_contract_version`
- For a broader description of what the published regex parser is expected to accept, consult the regex-flavor section in `PGEN_USER_GUIDE.md`.
- Full reporting procedure:
  - `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

## Minimal Rust Example
```rust
use pgen::embedding_api::{
    parse_regex_default_result, parser_embedding_api_contract,
};

let contract = parser_embedding_api_contract();
assert!(contract.supports_regex_generated_backend);
assert_eq!(contract.regex_parser_release_version, "1.1.0");

parse_regex_default_result(r"https?://[^\s]+")?;
```

## Validation / Release Gates
- Public host API stability:
  - `make -C rust embedding_api_gate`
  - `make -C rust regex_parser_integration_contract_gate`
- Embedded code-block structural contract:
  - `make -C rust regex_embedded_code_block_contract_gate`
- External corpus hardening:
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`
- Family proof/closure:
  - `make -C rust regex_parser_family_status_gate`
  - `make -C rust regex_parser_family_status_contract_gate`
  - `make -C rust regex_combined_telemetry_contract_gate`

## What This Does Not Promise
- It does not promise stable internal generated parser types.
- It does not promise runtime execution semantics for embedded code blocks such as `(?{...})`.
- It does not promise arbitrary valid Lua/JavaScript payload acceptance beyond the explicitly published structural forms.
- It does not promise `native` or `wasm` tagged embedded code blocks as part of the current published syntax.
- It does not promise every regex dialect already supported elsewhere in the ecosystem.
- It does not promise that downstream consumers can ignore parser release versioning when depending on AST details.
