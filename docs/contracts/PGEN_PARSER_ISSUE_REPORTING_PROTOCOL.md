# docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md

## Purpose
Define the exact reproduction bundle that downstream projects should provide when a PGEN-generated parser misbehaves after integration.

The goal is simple: make the first bug report precise enough that PGEN can reproduce the issue immediately, instead of spending time reconstructing the environment, the parser contract, or the missing input artifact.

Accepted reports should then be logged in:
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`

GitHub is optional. The required property is that both the report bundle and the resolution history are git-tracked somewhere stable, with PGEN retaining the canonical parser-side history.

## What A Good Report Must Contain

Version fields should be copied from the family's published handoff surface or stable metadata API, not guessed from memory or inferred from an old checkout note.

### 1. Parser Identity
- PGEN commit ID or released crate version.
- Parser release version, if the family publishes one.
  - for regex, copy `parser_embedding_api_contract().regex_parser_release_version`
- Integration contract version, if the family publishes one.
  - for regex, copy `parser_embedding_api_contract().regex_integration_contract_version`
- Parser family:
  - `systemverilog`
  - `vhdl`
  - `regex`
  - or the relevant annotation family if using annotation parsing directly
- Profile/backend in use:
  - for example `regex_default`, `sv_2023`, `vhdl_1076_2019`
- Integration surface in use:
  - `pgen::embedding_api`
  - `parseability_probe`
  - direct generated parser module
  - or a wrapped host adapter
- If the project vendors generated parser artifacts directly, include the exact generated file path and its provenance:
  - for example `generated/regex_parser.rs`

### 2. Host Project Identity
- Downstream project name.
- Downstream project commit ID or release version.
- Host OS and architecture.
- Rust toolchain version, if applicable.
- Enabled Cargo features or other build flags relevant to parser availability.

### 3. Exact Reproducer Input
- Attach the smallest exact input file that still reproduces the issue.
- Use a real text file, not a screenshot or a paraphrase.
- If the issue only occurs inside a larger host snippet, include:
  - the minimal failing file
  - and a short note describing how the host reached the parser call

### 4. Expected Vs Actual Behavior
- State exactly one of these bug classes:
  - should parse but fails
  - should fail but parses
  - parses but returns the wrong AST/dump
  - panics/crashes
  - pathological performance or resource usage
- Include the expected behavior and the actual observed behavior in plain language.

### 5. Structured PGEN Artifacts
- Contract metadata:
  - `parser_embedding_api_contract()` JSON for grammar-family integrations
- Parse outcome:
  - structured `parse_*` or `parse_grammar_profile_*` outcome JSON
  - if present, keep `ParseDiagnostic.location` intact:
    - `byte_offset`
    - `line`
    - `column`
- AST dump:
  - include when parse succeeds but the structure or semantics are wrong
- Trace log:
  - include when available using the protocol below

## Required Reproduction Procedure

### A. If The Downstream Project Can Run A PGEN Checkout
Set:
```bash
export PGEN_TRACE_VERBOSITY=debug
```

Then run the generated-parser probe with the exact failing input.

Support check:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --supports <grammar_name> \
  --profile <profile> \
  --trace \
  --trace-log-file pgen_trace.log
```

Parse check:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse <grammar_name> <input_file> \
  --profile <profile> \
  --trace \
  --trace-log-file pgen_trace.log
```

AST dump capture:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty <grammar_name> <input_file> pgen_ast_dump.json \
  --profile <profile> \
  --trace \
  --trace-log-file pgen_trace.log
```

Notes:
- `--trace` enables parser tracing.
- `PGEN_TRACE_VERBOSITY=debug` raises that trace to the most detailed built-in level.
- Replace:
  - `<grammar_name>` with `regex`, `systemverilog`, or `vhdl`
  - `<profile>` with the real profile in use such as `regex_default`, `sv_2023`, or `vhdl_1076_2019`

### B. If The Downstream Project Only Has The Embedded Rust API
Capture a tiny structured repro using `pgen::embedding_api`.

Recommended JSON bundle:
```rust
use pgen::embedding_api::{
    AstDumpOptions, parse_grammar_profile_ast_dump_named, parse_grammar_profile_named,
    parser_embedding_api_contract,
};

let contract = parser_embedding_api_contract();
let outcome = parse_grammar_profile_named("regex", "regex_default", &input);
let ast_dump = parse_grammar_profile_ast_dump_named(
    "regex",
    "regex_default",
    &input,
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);
```

Attach:
- pretty-printed `contract` JSON
- pretty-printed `outcome` JSON
- pretty-printed `ast_dump` JSON, when parse succeeds but the structure is wrong

### C. If The Failure Is A Panic Or Crash
Also capture:
```bash
export RUST_BACKTRACE=1
```

Attach the full backtrace log, not just the top line.

### D. If The Failure Is Performance-Related
Also include:
- input file byte size
- elapsed wall time
- whether trace was disabled for the measured run
- CPU/OS details
- whether the slowdown is:
  - deterministic on one sample
  - size-driven
  - or corpus-wide

## Recommended Artifact Names
- `repro_input.txt` or a grammar-appropriate extension
- `pgen_contract.json`
- `pgen_parse_outcome.json`
- `pgen_ast_dump.json`
- `pgen_trace.log`
- `host_call_site.md`

## What Not To Send
- screenshots instead of text artifacts
- edited or paraphrased input instead of the exact failing sample
- only the generated parser file without the input and structured outcome
- only a narrative such as “the parser seems wrong around groups” without a reproducer bundle

## Minimal Acceptable Report
A report is minimally actionable if it contains:
- exact PGEN version/commit
- parser release version, if published
- integration contract version, if published
- exact parser family/profile
- exact input file
- expected vs actual behavior
- structured parse outcome

It becomes high-quality and fast-to-fix if it also contains:
- `parser_embedding_api_contract()` JSON
- AST dump JSON
- `parseability_probe` trace log at `PGEN_TRACE_VERBOSITY=debug`

## After Intake
- Once a report is accepted as a real released-parser bug, PGEN should assign it a stable report ID, index it under the matching parser family/profile, and add it to `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
- The parser family/profile is the primary tracking axis for released-parser support.
  - Examples:
    - `REGEX-0001`
    - `VHDL-0003`
    - `SV-0012`
- If one or more downstream consumer repos also track the same issue locally, it is fine to record matching local entries there too.
- In that model:
  - PGEN should remain the canonical parser-side bug ledger and fix-proof owner
  - each downstream repo may track its own adoption, impact, local release notes, and release timing
- A fix should not be treated as complete until the ledger row records:
  - the root cause
  - the validating regression/gate proof
  - and the parser release carrying the fix, plus the commit when useful
