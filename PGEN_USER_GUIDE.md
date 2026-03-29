# PGEN User Guide

Last updated: 2026-03-25

## Current-State Companion Docs
- Use `README.md` as the main navigation and command-entry document.
- Use `RUST_CODEBASE_ANALYSIS.md` for the live Rust architecture/state assessment.
- Use `LIVE_ACHIEVEMENT_STATUS.md` for the current closure/status snapshot.
- Use `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` for the active roadmap contract.

## 1) What PGEN Is
PGEN is a parser/stimuli platform built around this flow:

1. `EBNF` grammar (`grammars/*.ebnf`)
2. `JSON` grammar artifact (`generated/*.json`)
3. Rust parser generation + AST tooling (`generated/*_parser.rs`, `rust/`)
4. Stimuli generation, coverage metrics, gap reports, differential checks, and CI gates

For annotation grammars specifically:

1. `grammars/return_annotation.ebnf` -> `generated/return_annotation.json` -> `generated/return_annotation_parser.rs`
2. `grammars/semantic_annotation.ebnf` -> `generated/semantic_annotation.json` -> `generated/semantic_annotation_parser.rs`

## 2) Core Concepts

### EBNF source grammars
- Human-authored grammars live under `grammars/`.
- Primary generated artifacts live under `generated/`.
- Regeneration overwrites generated artifacts.

### Return annotations
- Return annotations shape parse output ASTs for grammar rules.
- Main grammar: `grammars/return_annotation.ebnf`.
- Bootstrap/inferred grammar: `grammars/builtin_return_annotation.ebnf`.

### Semantic annotations
- Semantic annotations express metadata/transform intent used by the Rust AST pipeline.
- Main grammar: `grammars/semantic_annotation.ebnf`.
- Bootstrap/inferred grammar: `grammars/builtin_semantic_annotation.ebnf`.

### Bootstrap vs generated parser paths
- Bootstrap mode exists to break chicken-and-egg bootstrapping constraints.
- Generated parsers are the target steady-state behavior.
- Differential tooling tracks drift between the two.

### Quality gates (term definition)
A **gate** in PGEN is a hard, executable quality contract with pass/fail outcome that can block merge/release.

For a check to be considered a gate, it is captured in all of these places:
- local command entrypoint (usually a Make target in `rust/Makefile`)
- executable logic (`rust/scripts/*.sh`, `cargo test`, and/or `test_runner` suites)
- release policy classification (required vs informational in `rust/config/sota_exit_policy.env` and `rust/scripts/sota_exit_gate.sh`)
- CI execution (`.github/workflows/*.yml`)
- run evidence/artifacts (`rust/target/<gate_name>/...` logs/reports)

This keeps “gate” meaning aligned: it is not a guideline; it is an enforceable, versioned contract.

### Parseability (term definition)
A sample is **parseable** only when the matching generated parser accepts the **entire input** (full parse), not just a prefix.

In practical terms:
- parseability means `parse_full_*` success for the target grammar,
- prefix-only acceptance is not considered parseable,
- with `--validate-parseability`, only fully parseable generated samples are counted as accepted.
- `--parseability-report-json PATH` writes machine-readable parseability generation telemetry (`requested`, `accepted`, `rejected`, `attempts`, rejection breakdown, acceptance rate).
- `--parseability-max-attempts N` sets an explicit total attempt budget for parseability-aware generation instead of relying on the default `count * 50`.
- when parseability is driven by `--target-report-input`, the report may also include `target_drive_validation` with:
  - `primary_entry_attempts`
  - `primary_entry_accepted_outputs`
  - `primary_entry_rejected_outputs`
  - `primary_entry_acceptance_rate_percent`
  - `alternate_entry_attempts`
  - `alternate_entry_accepted_outputs`
  - `alternate_entry_rejected_outputs`
  - `alternate_entry_acceptance_rate_percent`
- the main target-driven quality gates now surface both primary-entry and alternate-entry counters in stage summaries or aggregate report artifacts so true entry-shaped rejection and helper-rule probe churn are both observable without opening raw JSON.
- aggregate `sota_exit_gate` summary now surfaces those primary-vs-alternate counters for:
  - `sv_preprocessor_quality`
  - `sv_stimuli_quality` replay-shadow telemetry
  - `vhdl_stimuli_quality` replay-shadow telemetry
- promotion-stage aggregate telemetry now surfaces both primary-entry and alternate-entry replay-shadow counters for:
  - `sv_declared_shadow_promotion` replay-shadow telemetry
  - `sv_parse_full_ratio_promotion` replay-shadow telemetry
  - `vhdl_strict_promotion` replay-shadow telemetry

## 3) Fast Start

### Build and run core gates
```bash
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash sota_release_policy
make -C rust SHELL=/bin/bash fixed_point_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
make -C rust SHELL=/bin/bash ci_workflow_local_gate
make -C rust SHELL=/bin/bash ebnf_stimuli_quality_gate
make -C rust SHELL=/bin/bash performance_gate
make -C rust SHELL=/bin/bash differential_regression_gate
make -C rust SHELL=/bin/bash embedding_api_gate
```

### Generate parser artifacts from annotation grammars
```bash
make -C rust SHELL=/bin/bash return_parser
make -C rust SHELL=/bin/bash semantic_parser
```

### Run universal tests
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- --parser return
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- --parser semantic
```

## 4) End-to-End Pipeline Commands

### EBNF -> JSON
```bash
tools/ebnf_to_json.pl --verbosity debug --pretty grammars/foolang.ebnf -o generated/foolang.json
```

Rust frontend alternative:
```bash
cargo run --manifest-path rust/Cargo.toml --features ebnf_dual_run --bin ast_pipeline -- \
  grammars/foolang.ebnf \
  --emit-raw-ast-json generated/foolang.json
```

### JSON -> parser source (Rust AST pipeline)
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- generated/foolang.json --generate-parser --output generated/foolang_parser.rs
```

### JSON/EBNF -> Rust stimuli module artifact
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- generated/foolang.json --generate-stimuli-module --count 128 --seed 7 --output generated/foolang_stimuli.rs
```

### SystemVerilog preprocess stage (raw SV -> expanded SV + metadata)
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  path/to/input.sv \
  --preprocess-systemverilog \
  --sv-include-dir path/to/includes \
  --sv-include-path-policy relative_only \
  --sv-macro-redefine-policy warn \
  --sv-conditional-symbol-policy assume_false_warn \
  --sv-conditional-expr-policy identifier_or_defined \
  --sv-strict-warning-codes W_SVPP_ABSOLUTE_INCLUDE_PATH,W_SVPP_UNSUPPORTED_CONDITIONAL_EXPR \
  --output /tmp/input.preprocessed.sv \
  --sv-source-map-json /tmp/input.preprocessed.map.json \
  --sv-event-log-json /tmp/input.preprocessed.events.json \
  --sv-diagnostics-json /tmp/input.preprocessed.diagnostics.json
```

### Existing Makefile pipeline shortcuts
```bash
make -C rust SHELL=/bin/bash return_annotation_parser
make -C rust SHELL=/bin/bash semantic_annotation_parser
make -C rust SHELL=/bin/bash regex_parser
```

## 4a) Local CI Workflow Parity

Use the local workflow parity gate before pushing when you want a clean-checkout approximation of the tracked GitHub workflow surface:

```bash
make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

Behavior:
- exports the current tracked worktree into a temporary local replay directory,
- excludes untracked files on purpose so local-only artifacts cannot hide CI failures,
- audits repo-local static `include!(...)` literals for absolute paths,
- verifies the tracked workflow/config/script surface required by the gate workflows,
- runs the same top-level gate commands used by the tracked `.github/workflows/*.yml` files.

Useful controls:
- run a subset of workflows:
  - `PGEN_CI_WORKFLOW_LOCAL_FILTER=annotation-contract-gate,branch-protection-contract-gate make -C rust SHELL=/bin/bash ci_workflow_local_gate`
- override local Cargo offline reuse:
  - `PGEN_CI_WORKFLOW_LOCAL_CARGO_OFFLINE=false make -C rust SHELL=/bin/bash ci_workflow_local_gate`

Compile-time include contract:
- repo-local `include!(...)` paths must stay relative,
- build-script-resolved HDL parser includes are emitted relative to `rust/src/` by `rust/build.rs`,
- clean local replay and GitHub workflow checkouts should not depend on absolute filesystem paths.

## 5) `ast_pipeline` CLI Guide

Reference:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- --help
```

Primary modes:

1. Transform JSON
- `ast_pipeline INPUT.json [OUTPUT.json]`

2. Export Rust EBNF `raw_ast`
- `ast_pipeline INPUT.ebnf --emit-raw-ast-json RAW.json`
- Requires building with `--features ebnf_dual_run`.

3. Generate parser
- `ast_pipeline INPUT.json --generate-parser --output PARSER.rs`

4. Generate stimuli
- `ast_pipeline INPUT.json --generate-stimuli --count N --seed S --output samples.txt`
- This is the default stimuli flow (in-memory generation, optional newline output artifact).

5. Generate stimuli module
- `ast_pipeline INPUT --generate-stimuli-module --count N [--seed S] [--output generated/<grammar>_stimuli.rs]`
- Emits a Rust file containing:
  - metadata constants (`STIMULI_MODULE_API_VERSION`, `GRAMMAR_NAME`, requested/generated sample count, seed, entry rule),
  - embedded `STIMULI` corpus,
  - `generated_stimuli()` accessor.
- Contract details:
  - default output path is `generated/<grammar>_stimuli.rs` when `--output` is omitted,
  - when `--seed` is omitted, module generation uses deterministic default seed `1`,
  - `ENTRY_RULE` is always resolved and exported as a concrete string constant,
  - module mode supports the same parseability/coverage/gap-report flags used by in-memory stimuli mode for parity and regression workflows.

6. Preprocess SystemVerilog source
- `ast_pipeline INPUT.sv --preprocess-systemverilog [--output PREPROCESSED.sv]`
- This mode executes the Rust preprocessor stage and emits:
  - expanded/preprocessed SV text,
  - optional source map JSON (`--sv-source-map-json`),
  - optional event log JSON (`--sv-event-log-json`),
  - optional diagnostics JSON (`--sv-diagnostics-json`) containing stable diagnostic tuples:
    - `code`, `severity`, `file`, `line`, `message`, `detail`.
- Deterministic controls:
  - include search path order via repeated `--sv-include-dir`,
  - bounded include recursion (`--sv-include-max-depth`),
  - include path policy (`--sv-include-path-policy` = `allow_absolute|relative_only`),
  - macro-redefinition policy (`--sv-macro-redefine-policy` = `allow|warn|error`),
  - conditional symbol policy (`--sv-conditional-symbol-policy` = `assume_false_silent|assume_false_warn|error`),
  - conditional expression policy (`--sv-conditional-expr-policy` = `identifier_only|identifier_or_defined`),
  - strict warning promotion (`--sv-strict-warning-codes` = `none|all|CSV of warning codes`).
- Backward-compatible strict macro mode:
  - `--sv-disallow-macro-redefine` forces macro redefinition policy to error.
- Strict warning promotion env fallback:
  - `PGEN_SVPP_STRICT_WARNING_CODES` is used when `--sv-strict-warning-codes` is omitted.

High-value stimuli flags:

- `--entry-rule`
- `--max-depth`
- `--max-repeat`
- `--dump-gen-ast [PATH]`
- `--dump-gen-ast-pretty`
- `--dump-gen-ast-max-bytes`
- `--recovery-stimuli-mode` (`baseline`, `recovery_biased`, `near_sync_negative`)
- `--enforce-word-boundary-spacing` (append delimiter spaces after terminal `\\b` regex samples to reduce merged-token outputs)
- `--validate-parseability`
- `--parseability-report-json`
- `--parseability-max-attempts`
- `--coverage-input`
- `--coverage-output`
- `--gap-report-json`
- `--gap-report-text`
- `--gap-report-threshold`
- `--target-report-input`
- `--gap-priority-report-input`
- `--target-max-attempts`
- `--coverage-guided-fuzz-rounds`
- `--coverage-guided-fuzz-seed-start`
- `--coverage-guided-fuzz-replay-output`

SystemVerilog preprocess flags:
- `--preprocess-systemverilog`
- `--sv-include-dir` (repeatable)
- `--sv-include-max-depth`
- `--sv-disallow-macro-redefine`
- `--sv-include-path-policy`
- `--sv-macro-redefine-policy`
- `--sv-conditional-symbol-policy`
- `--sv-conditional-expr-policy`
- `--sv-strict-warning-codes`
- `--sv-source-map-json`
- `--sv-event-log-json`
- `--sv-diagnostics-json`

### Tracing and Verbosity
Tracing in the Rust pipeline uses a single verbosity contract:
- `none`
- `low`
- `medium`
- `high`
- `debug`

CLI controls:
- `--verbosity <level>` controls how much trace is emitted.
- `--trace-log-file [PATH]` routes trace output to a file.

`--trace-log-file` behavior:
- `--trace-log-file` (no value) => writes trace to `trace.log`
- `--trace-log-file custom.log` => writes trace to `custom.log`
- when file routing is enabled, trace lines are written to the file instead of stdout.

Trace line origin contract:
- every trace line includes:
  - source file name,
  - function name,
  - source line number.
- header format:
  - `[PGEN][<LEVEL>] ... [<file>:<line>] [<function>] <message>`

Environment controls:
- `PGEN_TRACE_VERBOSITY` (fallback: `PGEN_VERBOSITY`)
- `PGEN_TRACE_LOG_FILE`

### Generation-Input AST Dump (Phase R Item 1)
Use the generation-input AST dump when you need to inspect the exact normalized AST consumed by parser/stimuli generators.

CLI controls:
- `--dump-gen-ast [PATH]`
  - writes JSON dump of:
    - `grammar_name`,
    - `rule_order`,
    - `grammar_tree`,
    - `annotations`.
  - if flag is present with no value, default path is `gen_ast.json`.
- `--dump-gen-ast-pretty`
  - pretty JSON formatting mode (requires `--dump-gen-ast`).
- `--dump-gen-ast-max-bytes <N>`
  - bounds output file size for generation-input AST dump.
  - when encoded AST JSON exceeds `N`, tool writes a deterministic truncation-diagnostics JSON envelope instead of full AST payload.
  - `N` must be `>= 1`.

Mode contract:
- `--dump-gen-ast` is valid only with generation modes:
  - `--generate-parser`
  - `--generate-stimuli`
  - `--generate-stimuli-module`
- optional env fallback:
  - `PGEN_DUMP_GEN_AST_MAX_BYTES` (used when `--dump-gen-ast-max-bytes` is omitted).

Determinism and safety contract:
- dump JSON object keys are canonicalized recursively for deterministic key-order output.
- bounded-size safeguard:
  - if encoded AST JSON size `>` configured max-bytes, output is:
    - `kind = "pgen_ast_dump_truncation"`
    - `truncated = true`
    - `dump_kind = "generation_input_ast"`
    - `max_bytes`
    - `full_bytes`
    - `reason`
  - if max-bytes is too small to fit truncation diagnostics envelope, command fails with explicit error.

Examples:
```bash
# Default dump path (gen_ast.json) while generating parser
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/json.json \
  --generate-parser \
  --output /tmp/json_parser.rs \
  --dump-gen-ast

# Explicit dump path + pretty JSON while generating stimuli
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/json.json \
  --generate-stimuli \
  --count 8 \
  --seed 7 \
  --dump-gen-ast /tmp/gen_ast.json \
  --dump-gen-ast-pretty \
  --output /tmp/json_stimuli.txt

# Enforce bounded dump output and emit truncation diagnostics when oversized
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/json.json \
  --generate-parser \
  --output /tmp/json_parser.rs \
  --dump-gen-ast /tmp/gen_ast.json \
  --dump-gen-ast-max-bytes 4096
```

### Parser-Returned AST Dump (`parseability_probe`)
Use parser-returned AST dump when you need the parsed AST produced by generated parsers for a concrete input sample.

CLI controls:
- `parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file]`
- `parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file]`
- optional tail flag for both commands:
  - `--max-bytes <N>`

Default filename contract:
- when `output_file` is omitted, dump file is:
  - `<grammar_name>_ast.json`
- examples:
  - `foolang_ast.json`
  - `ebnf_ast.json`
  - `regex_ast.json`
  - `vhdl_ast.json`
  - `systemverilog_ast.json`

Determinism and safety contract:
- parser-returned AST dump JSON object keys are canonicalized recursively for deterministic replay/diff output.
- bounded-size safeguard:
  - `--max-bytes <N>` bounds parser AST dump output size.
  - env fallback:
    - `PGEN_PARSE_DUMP_AST_MAX_BYTES` (used when `--max-bytes` is omitted).
  - if encoded AST JSON size exceeds configured bound, output file is replaced with deterministic truncation diagnostics envelope:
    - `kind = "pgen_ast_dump_truncation"`
    - `truncated = true`
    - `dump_kind = "parser_return_ast"`
    - `max_bytes`
    - `full_bytes`
    - `reason`
  - if configured bound is too small to fit diagnostics envelope, command fails explicitly.

Examples:
```bash
# Parse and dump AST to default grammar-based path (ebnf_ast.json)
cargo run --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin parseability_probe -- \
  --parse-dump-ast ebnf /tmp/sample.ebnf

# Parse and dump pretty AST to explicit output file
cargo run --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin parseability_probe -- \
  --parse-dump-ast-pretty ebnf /tmp/sample.ebnf /tmp/custom_ebnf_ast.json

# Parse and enforce bounded output size (truncation envelope emitted when oversized)
cargo run --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin parseability_probe -- \
  --parse-dump-ast builtin_semantic_annotation /tmp/sample.sem /tmp/semantic_ast.json --max-bytes 4096
```

### Parser-Returned AST Dump (Embedding API)
For host integrations (for example Nexsim), use the embedding API AST-dump surface to get parser-returned AST JSON directly in memory.

Stable Rust APIs:
- `parse_grammar_profile_ast_dump(...)`
- `parse_grammar_profile_ast_dump_with_limits(...)`
- `parse_grammar_profile_ast_dump_result(...)`
- `parse_grammar_profile_ast_dump_with_limits_result(...)`
- `parse_grammar_profile_ast_dump_named(...)`
- `parse_grammar_profile_ast_dump_named_with_limits(...)`

Convenience profile APIs:
- `parse_systemverilog_2017_ast_dump(...)`
- `parse_systemverilog_2023_ast_dump(...)`
- `parse_vhdl_1076_2019_ast_dump(...)`
- plus `*_ast_dump_with_limits(...)` variants.

AST dump controls:
- `AstDumpOptions { pretty, max_ast_bytes }`
  - `pretty=false` (default): compact canonical JSON
  - `pretty=true`: canonical pretty JSON
  - `max_ast_bytes=None` (default): unbounded
  - `max_ast_bytes=Some(N)`: bounded output (`N >= 1`)

Payload contract:
- On success, API returns `AstDumpPayload`:
  - `dump_json`
  - `truncated`
  - `full_bytes`
  - `emitted_bytes`
- If bounded output truncates payload, `dump_json` contains deterministic truncation diagnostics JSON envelope with:
  - `kind = "pgen_ast_dump_truncation"`
  - `dump_kind = "parser_return_ast"`
  - `max_bytes`
  - `full_bytes`
  - `reason`

### AST Debug Playbooks (SV/VHDL/Regex)
Use this sequence to debug grammar intent (`*.ebnf`) versus generated behavior with deterministic artifacts.

Shared principles:
- `gen_ast.json` answers: "what AST did codegen consume?"
- `<grammar>_ast.json` answers: "what AST did parser return for this sample?"
- diffing both surfaces is the fastest way to isolate return-annotation/codegen mismatches.

#### A) SystemVerilog triage flow (Nexsim onboarding)
1. Dump generation-input AST while generating parser:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/systemverilog.json \
  --generate-parser \
  --output /tmp/systemverilog_parser.rs \
  --dump-gen-ast /tmp/systemverilog_gen_ast.json \
  --dump-gen-ast-pretty
```
2. Dump parser-returned AST for a real sample:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty systemverilog /tmp/sample.sv /tmp/systemverilog_ast.json
```
3. Enforce bounded dump contract in stress/debug runs:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast systemverilog /tmp/sample.sv /tmp/systemverilog_ast.json --max-bytes 65536
```
4. Optional embedding-side AST dump (in-memory host integration):
```rust
use pgen::embedding_api::{
    AstDumpOptions, parse_systemverilog_2023_ast_dump,
};

let outcome = parse_systemverilog_2023_ast_dump(
    sample_text,
    &AstDumpOptions { pretty: true, max_ast_bytes: Some(65_536) },
);
```

#### B) VHDL triage flow
1. Dump generation-input AST:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/vhdl.json \
  --generate-parser \
  --output /tmp/vhdl_parser.rs \
  --dump-gen-ast /tmp/vhdl_gen_ast.json \
  --dump-gen-ast-pretty
```
2. Dump parser-returned AST:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty vhdl /tmp/sample.vhd /tmp/vhdl_ast.json
```
3. If parse fails, capture deterministic parseability result first:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse vhdl /tmp/sample.vhd
```

#### C) Regex grammar onboarding/debug flow
Current note:
- parser-returned AST dump via `parseability_probe` is adapter-based and should be treated as available only when the grammar is registered.
- generation-input AST dump is always available for regex codegen/stimuli triage.

1. Dump generation-input AST:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/regex.json \
  --generate-parser \
  --output /tmp/regex_parser.rs \
  --dump-gen-ast /tmp/regex_gen_ast.json \
  --dump-gen-ast-pretty
```
2. Drive stimuli + coverage/gap for deterministic regex hardening:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/regex.json \
  --generate-stimuli \
  --count 64 \
  --seed 2026 \
  --coverage-output /tmp/regex_coverage.json \
  --gap-report-json /tmp/regex_gap.json \
  --output /tmp/regex_stimuli.txt
```
3. Re-run with `--target-coverage-json /tmp/regex_gap.json` to verify gap-driven convergence.

### Tracing Examples
```bash
# Debug trace to default trace.log
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/json.json \
  --generate-stimuli \
  --count 8 \
  --verbosity debug \
  --trace-log-file

# Medium trace to explicit file path
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- \
  generated/json.json \
  --generate-parser \
  --output generated/json_parser.rs \
  --verbosity medium \
  --trace-log-file /tmp/pgen_trace.log
```

Important:
- Parseability validation currently supports generated parser checks for:
  - `ebnf` (requires `generated/ebnf.rs` and building with `--features "generated_parsers ebnf_dual_run"`)
  - `return_annotation`
  - `semantic_annotation`
  - `builtin_return_annotation`
  - `builtin_semantic_annotation`
- Parseability checks require building with generated parsers:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- ...
```
- For `ebnf` parseability checks, use:
```bash
cargo run --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin ast_pipeline -- ...
```

### In-Memory vs Module Mode (When to Use Which)
- Use `--generate-stimuli` when:
  - you need quick ad-hoc generation,
  - you only need newline output or direct stdout samples,
  - you do not need a reusable Rust artifact.
- Use `--generate-stimuli-module` when:
  - you need a reusable, checked-in/attached Rust artifact for embedding,
  - you want explicit metadata constants (`SEED`, `ENTRY_RULE`, counts) in the artifact,
  - you need deterministic replay defaults even when caller omits `--seed` (defaults to `1` in module mode).

### Embedding Workflow Examples (`generated/<grammar>_stimuli.rs`)
1. Generate module from EBNF (frontend + module artifact):
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  grammars/semantic_annotation.ebnf \
  --generate-stimuli-module \
  --count 64 \
  --seed 7201 \
  --validate-parseability \
  --coverage-output /tmp/semantic_cov.json \
  --gap-report-json /tmp/semantic_gap.json \
  --output generated/semantic_annotation_stimuli.rs
```

2. Consume generated module in Rust (recommended import style):
```rust
#[path = "../generated/semantic_annotation_stimuli.rs"]
mod semantic_annotation_stimuli;

fn main() {
    assert_eq!(semantic_annotation_stimuli::STIMULI_MODULE_API_VERSION, 1);
    for sample in semantic_annotation_stimuli::generated_stimuli() {
        println!("{}", sample);
    }
}
```

3. Replay the exact same corpus using in-memory mode:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 64 \
  --seed 7201 \
  --entry-rule semantic_annotation \
  --validate-parseability \
  --parseability-max-attempts 3200 \
  --parseability-report-json /tmp/semantic_parseability.json \
  --output /tmp/semantic_replay.txt
```

With `--validate-parseability`, add `--parseability-report-json PATH` when you want structured acceptance-effort telemetry for the run rather than only the human-readable summary line on stdout. Add `--parseability-max-attempts N` when a gate or experiment needs an explicit acceptance-effort budget.

When `--validate-parseability` is combined with `--target-report-input`, target-driven generation is validator-aware:
- parser-rejected outputs do not pay down rule/branch success debt,
- rejected outputs are excluded from returned samples,
- branch-selection history is still retained so the generator can throttle repeatedly failing target branches instead of forgetting them.
- once a branch has enough history, that throttle is driven by accepted-success yield (`selected_counts` versus `success_counts`), so branches that are selected often and rarely accepted are de-emphasized generically instead of only clamping zero-success branches.
- when alternate non-entry probes become both dominant and low-yield, validator-backed replay backs off generic non-entry probing and increases probe threshold so remaining budget is spent more on primary-entry validation than helper-rule churn.
- `--parseability-report-json` now also includes `target_drive_validation` for this mode, with:
  - `primary_entry_attempts`
  - `primary_entry_accepted_outputs`
  - `primary_entry_rejected_outputs`
  - `primary_entry_acceptance_rate_percent`
  - `alternate_entry_attempts`
  - `alternate_entry_accepted_outputs`
  - `alternate_entry_rejected_outputs`
  - `alternate_entry_acceptance_rate_percent`
  so you can see how much parser-backed target closure was spent on non-entry probe rules versus real entry-shaped outputs.

### Deterministic Replay and Seed Compatibility Guarantees
- In-memory mode (`--generate-stimuli`):
  - deterministic only when `--seed` is explicitly provided,
  - without `--seed`, generation uses entropy (`StdRng::from_entropy`) and is not replay-stable.
- Module mode (`--generate-stimuli-module`):
  - deterministic with explicit `--seed`,
  - deterministic with omitted `--seed` due to enforced default seed `1`.
- Cross-mode replay compatibility requires matching all of:
  - grammar content + grammar name,
  - `entry_rule`,
  - `count`,
  - `seed`,
  - `max_depth`,
  - `max_repeat`,
  - `recovery_stimuli_mode`,
  - parseability filter setting,
  - parseability attempt budget (when parseability validation is enabled),
  - coverage merge input (if used).
- Cross-mode parity (samples + coverage + gap) is enforced by:
```bash
make -C rust SHELL=/bin/bash stimuli_module_parity_gate
```

## 6) Coverage and Gap Workflows

### A) Baseline stimuli + coverage output
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 200 \
  --seed 42 \
  --output /tmp/semantic_samples.txt \
  --coverage-output /tmp/semantic_coverage.json
```

### B) Merge previous coverage + produce gap report
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 200 \
  --coverage-input /tmp/semantic_coverage.json \
  --coverage-output /tmp/semantic_coverage_merged.json \
  --gap-report-json /tmp/semantic_gap.json \
  --gap-report-text /tmp/semantic_gap.txt
```

### C) Target-driven closure
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --target-report-input /tmp/semantic_gap.json \
  --target-max-attempts 8000 \
  --output /tmp/semantic_targeted.txt
```

### D) Gap-priority count-based generation
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 300 \
  --gap-priority-report-input /tmp/semantic_gap.json \
  --output /tmp/semantic_priority.txt
```

### E) Coverage-guided fuzz with replay + corpus minimization
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --coverage-guided-fuzz-rounds 200 \
  --coverage-guided-fuzz-seed-start 1000 \
  --coverage-guided-fuzz-replay-output /tmp/semantic_fuzz_replay.json \
  --output /tmp/semantic_fuzz_minimized.txt
```

### F) Recovery-focused and near-sync negative-case stimuli modes
```bash
# Recovery-biased samples (inject recovery marker context around generated samples)
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 50 \
  --recovery-stimuli-mode recovery_biased \
  --output /tmp/recovery_biased_samples.txt

# Near-sync negative-case samples (inject deterministic invalid noise adjacent to marker)
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 50 \
  --recovery-stimuli-mode near_sync_negative \
  --output /tmp/near_sync_negative_samples.txt
```

## 7) Return Annotation Features

Main grammar: `grammars/return_annotation.ebnf`

Common constructs:

- Positional capture reference:
  - `-> $1`
- Spread:
  - `-> $2*`
- Quantified extraction:
  - `-> $2::first`
  - `-> $2::2*`
- Property access:
  - `-> $1.value`
- Array access:
  - `-> $1[0]`
- Object shaping:
  - `-> {kind: "node", lhs: $1, rhs: $3}`
- Array shaping:
  - `-> [$1, $2::1*]`
- String literal parity:
  - `-> 'node'`
  - `-> "node"`
- Identifier literal:
  - `-> node_kind`

Bootstrap behavior notes:
- Bootstrap parser is intentionally permissive and has known quirks.
- For implementation-accurate bootstrap behavior, see:
  - `grammars/builtin_return_annotation.ebnf`
  - `rust/src/ast_pipeline/unified_return_ast.rs`

## 8) Semantic Annotation Features

Main grammar: `grammars/semantic_annotation.ebnf`

### 8.1 Syntax Model
- Preferred semantic form is named directive:
  - `@name: payload`
  - examples: `@type: "Expression"`, `@precedence: 5`, `@range: 0..10`
- Payloads may be scalar, list, or expression depending on directive.
- Legacy/raw semantic payloads are still accepted for compatibility, but typed named directives are the contract surface for steering behavior.

### 8.2 Bootstrap vs Generated Semantic Parsers
- Bootstrap mode is intentionally limited/permissive and exists only to break the chicken-and-egg dependency for annotation parser generation.
- Generated parser is the long-term behavior target.
- Bootstrap references:
  - `grammars/builtin_semantic_annotation.ebnf`
  - `rust/src/ast_pipeline/unified_semantic_ast.rs`

### 8.3 Typed Directive Routing + Unknown Directive Policy
- Directive names are resolved through the typed registry in `rust/src/ast_pipeline/semantic_directive_registry.rs`.
- Unknown directive handling is policy-driven:
  - `PGEN_UNKNOWN_SEMANTIC_DIRECTIVE_POLICY=ignore|warn|strict` (default `warn`).
  - `warn`: emits `W_SEM_UNKNOWN_DIRECTIVE`.
  - `strict`: promotes unknown directive to `error`.
- Strict warning-promotion policy is explicit:
  - `PGEN_STRICT_SEMANTIC_WARNING_CODES=<comma-separated-codes|all|none>`
  - applies when strict annotation validation is enabled (`CI` default, or `PGEN_STRICT_ANNOTATION_VALIDATION=1`),
  - `all` (or `*`) promotes all semantic warning-class diagnostics to errors,
  - `none` disables warning-code promotion while keeping strict validation enabled.
- Strict default warning promotions (when strict validation is enabled and no explicit warning policy is set):
  - `W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`
  - `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`
- Directive routing is name-aware:
  - transform steering is only active for directive `transform`,
  - raw literal hint steering is only active for literal/sample directive family in named mode.

#### 8.3.1 SC-03 Tier-4 Contract Gate

SC-03 routing/strictness behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc03_contract.json`

Gate commands:
```bash
make -C rust sc03_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc03_contract_gate` enforces:
1. Typed directive registry + capability taxonomy contracts.
2. Unknown-directive warn/strict policy validator contracts.
3. Strict warning-code selection policy contracts.
4. Parser/stimuli name-aware transform/literal routing guard tests.
5. Bootstrap/generated SC-03 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-03 comparable corpus).

#### 8.3.2 SC-06 Tier-4 Contract Gate

SC-06 branch weighting/selection behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc06_contract.json`

Gate commands:
```bash
make -C rust sc06_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc06_contract_gate` enforces:
1. Typed branch-policy payload contracts (`W_SEM_INVALID_BRANCH_POLICY_PAYLOAD` + valid payload acceptance).
2. Parser codegen branch-policy routing contract coverage.
3. Stimuli branch-selection behavior for `ordered` and `priority_first`.
4. Weighted-probability deterministic sampling and equal-weight fallback contracts.
5. Bootstrap/generated SC-06 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-06 comparable corpus).

#### 8.3.3 SC-07 Tier-4 Contract Gate

SC-07 recovery/sync behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc07_contract.json`

Gate commands:
```bash
make -C rust sc07_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc07_contract_gate` enforces:
1. Typed recovery payload/coherence validator contracts (`@recover/@recover_budget/@recover_parse_budget/@recover_global_budget/@sync/@panic_until`).
2. Parser codegen recovery-policy extraction and runtime recovery hook emission guard contracts.
3. Generated-parser structured recovery telemetry surface contracts (`recovery_events` + counters/accessors).
4. Stimuli recovery fallback and recovery-focused mode contracts (`recovery_biased`, `near_sync_negative`).
5. Bootstrap/generated SC-07 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-07 comparable corpus).

#### 8.3.4 SC-09 Tier-4 Contract Gate

SC-09 relational constraint behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc09_contract.json`

Gate commands:
```bash
make -C rust sc09_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc09_contract_gate` enforces:
1. Typed relational payload parser contracts (`@constraint/@requires/@implies`).
2. Typed relational validator/coherence contracts (`W_SEM_INVALID_*` and `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT` behavior).
3. Parser codegen/runtime relational guard contracts (policy extraction + helper/runtime guard surfaces).
4. Stimuli relational steering contracts including nested structured/non-structured reference paths and ranked unsatisfiable diagnostics.
5. Bootstrap/generated SC-09 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-09 comparable corpus).

#### 8.3.5 SC-10 Tier-4 Contract Gate

SC-10 coverage-target behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc10_contract.json`

Gate commands:
```bash
make -C rust sc10_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc10_contract_gate` enforces:
1. Typed SC-10 payload parser contracts (`@coverage_target/@critical_path`).
2. Typed SC-10 validator/coherence contracts including strict-warning policy behavior and `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`.
3. Parser codegen/runtime SC-10 instrumentation contracts (`CoverageTargetEvent` and accessors/hit counters).
4. Stimuli SC-10 coverage steering contracts (branch sampling bias and gap-priority target ordering bonuses).
5. Bootstrap/generated SC-10 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-10 comparable corpus).

#### 8.3.6 SC-11 Tier-4 Contract Gate

SC-11 negative-case behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc11_contract.json`

Gate commands:
```bash
make -C rust sc11_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc11_contract_gate` enforces:
1. Typed SC-11 payload parser contracts (`@invalid_case/@negative`).
2. Typed SC-11 validator/coherence contracts (`W_SEM_INVALID_INVALID_CASE_PAYLOAD`, `W_SEM_INVALID_NEGATIVE_PAYLOAD`, `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE`).
3. Parser codegen/runtime negative-case instrumentation contracts (`NegativeCaseEvent` and accessors/hit counters).
4. Stimuli invalid/negative generation contracts (invalid mutation + negative marker guard behavior).
5. Bootstrap/generated SC-11 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-11 comparable corpus).

#### 8.3.7 SC-12 Tier-4 Contract Gate

SC-12 deterministic-partition behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc12_contract.json`

Gate commands:
```bash
make -C rust sc12_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc12_contract_gate` enforces:
1. Typed SC-12 payload parser contracts (`@seed_group/@deterministic_group`).
2. Typed SC-12 validator/coherence contracts (`W_SEM_INVALID_SEED_GROUP_PAYLOAD`, `W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD`, `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP`).
3. Parser deterministic-partition runtime contracts (policy extraction, runtime mode/accessor surface, runtime-order branch partitioning and event recording).
4. Stimuli deterministic-partition contracts (inactive seed-group guard, deterministic-group routing, and interleaving-order independence).
5. Bootstrap/generated SC-12 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-12 comparable corpus).

#### 8.3.8 SC-05 Tier-4 Contract Gate

SC-05 precedence/associativity behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc05_contract.json`

Gate commands:
```bash
make -C rust sc05_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc05_contract_gate` enforces:
1. Typed SC-05 payload parser contracts (`@priority/@precedence/@associativity`).
2. Typed SC-05 validator/coherence contracts (`W_SEM_INVALID_PRIORITY_PAYLOAD`, `W_SEM_INVALID_ASSOCIATIVITY_PAYLOAD`, `W_SEM_PRIORITY_PRECEDENCE_CONFLICT`, duplicate last-wins diagnostics).
3. Parser precedence/associativity runtime contracts (directive extraction, priority-over-precedence routing, associativity tie-break policy emission).
4. Stimuli precedence/associativity steering contracts (priority bias, priority-over-precedence steering, and associativity tie biasing behavior).
5. Bootstrap/generated SC-05 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-05 comparable corpus).

#### 8.3.9 SC-08 Tier-4 Contract Gate

SC-08 value-domain behavior is gate-enforced.

Contract corpus:
- `rust/test_data/semantic_annotation/sc08_contract.json`

Gate commands:
```bash
make -C rust sc08_contract_gate
```

Also included in:
```bash
make -C rust annotation_contract_gate
```

`sc08_contract_gate` enforces:
1. Typed SC-08 payload parser contracts (`@range/@enum/@len/@regex`).
2. Typed SC-08 validator/coherence contracts (invalid value-domain payload diagnostics + unsatisfiable intersection diagnostics).
3. Parser value-domain runtime contracts (value-constraint guard emission for regex atoms and numeric range guards).
4. Stimuli value-domain steering contracts (enum/range/len/regex filtering and composed-constraint synthesis behavior).
5. Bootstrap/generated SC-08 semantic suite parity + differential taxonomy integrity checks (`mismatched_cases == 0` for SC-08 comparable corpus).

### 8.4 Steering Behaviors Implemented Today

#### A) Canonical transform steering (`@transform`)
- Canonical form:
  - `str::parse::<T>().unwrap_or(default)`
- Parser behavior:
  - for regex atoms, generated parser emits transformed terminal output for canonical transform.
  - target type supports path forms (for example `std::primitive::i64`).
- Stimuli behavior:
  - canonical transform target type can drive deterministic hint value:
    - integer targets -> `"1"`
    - float targets -> `"1.0"`
    - bool target -> `"true"`
- Non-canonical transform expressions are accepted but do not trigger canonical typed steering.

#### B) Branch steering (`@priority`, `@precedence`, `@associativity`)
- `@priority` / `@precedence` payloads:
  - scalar: `5` (applies to all OR branches),
  - list: `[1, 9, 2]` (applies by OR-branch index).
- `@associativity` payloads:
  - `left`, `right`, `nonassoc`.
- Conflict-resolution contract:
  - if both `@priority` and `@precedence` are present for a rule, `@priority` deterministically overrides `@precedence` regardless of annotation order.
  - repeated occurrences of a known directive follow deterministic last-wins behavior.
- Parser OR tie-break behavior:
  - primary remains longest-match,
  - semantic priority/precedence breaks equal-length ties,
  - associativity controls exact-equality resolution:
    - `left`: keep earlier winner,
    - `right`: prefer later winner,
    - `nonassoc`: unresolved exact ties backtrack.
- Stimuli OR branch behavior:
  - semantic priority/precedence contributes branch sampling bias,
  - associativity biases equal structures toward earlier/later branch order.

#### C) Token-family steering (`@token_class`, `@charset`, `@pattern`)
- Intended scope:
  - steering applies to regex atoms in a rule,
  - steering is inactive for rules with no regex atom (validator emits `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`).
- Payload forms:
  - `@token_class`: known class label (for example `identifier`, `int`, `float`, `bool`, `word`, `alnum`, `lower`, `upper`, `whitespace`, `hex`, `binary`, `printable`).
  - `@charset`: character-class payload (for example `A-Za-z_` or `[0-9A-F]`).
  - `@pattern`: full regex payload.
- Deterministic precedence (when multiple are present):
  1. `@pattern`
  2. `@charset`
  3. `@token_class`
- Validator diagnostics:
  - payload diagnostics:
    - `W_SEM_INVALID_TOKEN_CLASS_PAYLOAD`
    - `W_SEM_INVALID_CHARSET_PAYLOAD`
    - `W_SEM_INVALID_PATTERN_PAYLOAD`
  - overlap diagnostic:
    - `W_SEM_TOKEN_STEERING_PRECEDENCE` (documents precedence contract when multiple steering directives appear together).
- Runtime behavior:
  - parser: generated regex matcher uses effective steering pattern after precedence resolution,
  - stimuli: regex sample generation uses the same effective steering pattern after precedence resolution.

#### D) Value-domain steering (`@enum`, `@range`, `@len`, `@regex`)
- `@enum` payload forms:
  - `["AA", "BB"]`
  - `"AA"` (single value form also accepted).
- `@range` payload forms:
  - `10..20`
  - `[10, 20]`
  - `7` (exact-value shorthand).
- `@len` payload forms:
  - `2..8`
  - `[2, 8]`
  - `4` (exact-length shorthand).
- `@regex` payload:
  - non-empty regex pattern string/scalar.
- Deterministic merge/precedence behavior:
  - different value-domain directives compose conjunctively (all active constraints must pass),
  - repeated occurrences of the same directive currently follow last-wins assignment per rule.
  - when `@enum` is present with `@len`/`@range`/`@regex`, validator checks satisfiability of the intersection and warns if no enum value can satisfy all active constraints.

#### E) Relational constraint contract baseline (`@constraint`, `@requires`, `@implies`)
- Current stage:
  - typed validator contract is implemented,
  - parser runtime baseline (`Tier 2`) is implemented,
  - stimuli runtime baseline (`Tier 3`) is implemented for root sequence synthesis retries.
- Payload expectations:
  - `@constraint`: non-empty expression/scalar payload.
  - `@requires`: one or more references (for example `["$1", "lhs.id"]`).
  - `@implies`: implication expression using `=>` (for example `"$1 => $2"`).
- Parser runtime behavior (when `@constraint` is present):
  - `@requires`: each reference must resolve and be non-empty.
  - `@constraint`: relational expression is evaluated against resolved capture/reference values.
  - `@implies`: antecedent must not evaluate to true while consequent is false.
  - unresolved references and failed checks return contextual parse errors.
- Stimuli runtime behavior (when `@constraint` is present on a rule whose root is a sequence):
  - generator retries sequence synthesis until relational checks pass or attempt budget is exhausted,
  - same contracts are enforced (`@requires`, `@constraint`, `@implies`),
  - stimuli reference support covers positional and named nested paths (for example `$1.id`, `lhs.id`, `$3.id.len`) over structured JSON-like captures and non-structured object-like captures (for example `id=AA,meta.kind=lhs`), with optional `.len`.
- Coherence rule:
  - `@requires`/`@implies` without `@constraint` triggers `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`.

#### F) Coverage-target steering contract baseline (`@coverage_target`, `@critical_path`)
- Current stage:
  - typed validator contract is implemented,
  - stimuli coverage/gap steering baseline (`Tier 3` coverage pipeline) is implemented,
  - parser runtime instrumentation baseline is implemented (`CoverageTargetEvent` + rule/branch hit counters).
- Payload expectations:
  - `@coverage_target`: non-negative integer or boolean payload (for example `3`, `true`, `false`).
  - `@critical_path`: boolean payload (for example `true`/`false`).
- Validator behavior:
  - invalid payloads emit:
    - `W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`
    - `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`
  - coherence warning:
    - `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET` when `@critical_path` is enabled but effective `@coverage_target` is missing/zero.
- Stimuli behavior:
  - branch guidance multipliers are boosted for SC-10-tagged rules,
  - branches referencing SC-10-tagged rules are also boosted,
  - gap-report target priorities include semantic SC-10 bonus scores for rule and branch debt ordering.
- Parser instrumentation behavior:
  - successful targeted-rule parses emit `CoverageTargetEvent`,
  - OR rules tag selected branch index in the event,
  - generated parser exposes:
    - `coverage_target_events()`
    - `take_coverage_target_events()`
    - `coverage_target_event_count()`
    - `coverage_target_rule_hits()`
    - `coverage_target_branch_hits()`
  - instrumentation is inactive when effective `@coverage_target` weight is zero.

### 8.5 Parser Runtime Contract for Value-Domain Directives
- Value-domain guards are injected into generated parser code for relevant atom token paths:
  - `quoted_string`
  - `regex`
  - `number`, `probability`, `include_dir`, `include_file`, `rule`
- Guard order:
  1. enum membership
  2. semantic regex full-match check
  3. length bounds
  4. numeric bounds
- In canonical transform paths, guard checks are applied before type transform.
- On violation, generated parser returns contextual parse error with semantic constraint reason.

### 8.6 Stimuli Runtime Contract for Value-Domain Directives
- Regex-token sample selection order:
  1. semantic hint (only if it satisfies active constraints),
  2. enum candidates filtered by grammar regex and active constraints,
  3. constraint-driven candidate (numeric range or length candidate),
  4. regex HIR generation attempts (bounded retries) filtered by constraints,
  5. deterministic fallback.
- Constraint satisfaction is conjunction-based:
  - enum + regex + len + range all must be satisfied if present.

### 8.7 Typed Validator Diagnostics (Semantic Directives)
- Unknown directive:
  - `W_SEM_UNKNOWN_DIRECTIVE` (policy-dependent severity).
- Transform diagnostics:
  - `W_SEM_NON_CANONICAL_TRANSFORM`
  - `W_SEM_DEFAULT_TYPE_MISMATCH`
  - `W_SEM_UNKNOWN_TARGET_TYPE`
- Value/branch payload diagnostics:
  - `W_SEM_INVALID_ASSOCIATIVITY_PAYLOAD`
  - `W_SEM_INVALID_PRIORITY_PAYLOAD`
  - `W_SEM_INVALID_BRANCH_POLICY_PAYLOAD`
  - `W_SEM_INVALID_RECOVER_PAYLOAD`
  - `W_SEM_INVALID_RECOVER_BUDGET_PAYLOAD`
  - `W_SEM_INVALID_RECOVER_PARSE_BUDGET_PAYLOAD`
  - `W_SEM_INVALID_RECOVER_GLOBAL_BUDGET_PAYLOAD`
  - `W_SEM_INVALID_CONSTRAINT_PAYLOAD`
  - `W_SEM_INVALID_REQUIRES_PAYLOAD`
  - `W_SEM_INVALID_IMPLIES_PAYLOAD`
  - `W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`
  - `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`
  - `W_SEM_INVALID_INVALID_CASE_PAYLOAD`
  - `W_SEM_INVALID_NEGATIVE_PAYLOAD`
  - `W_SEM_INVALID_SEED_GROUP_PAYLOAD`
  - `W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD`
  - `W_SEM_INVALID_TOKEN_CLASS_PAYLOAD`
  - `W_SEM_INVALID_CHARSET_PAYLOAD`
  - `W_SEM_INVALID_PATTERN_PAYLOAD`
  - `W_SEM_INVALID_SYNC_PAYLOAD`
  - `W_SEM_INVALID_PANIC_UNTIL_PAYLOAD`
  - `W_SEM_INVALID_ENUM_PAYLOAD`
  - `W_SEM_INVALID_RANGE_PAYLOAD`
  - `W_SEM_INVALID_LEN_PAYLOAD`
  - `W_SEM_INVALID_REGEX_PAYLOAD`
- Conflict diagnostics:
  - `W_SEM_PRIORITY_PRECEDENCE_CONFLICT` (`@priority` + `@precedence` both present; `@priority` takes precedence)
  - `W_SEM_DIRECTIVE_OVERRIDDEN` (same known directive repeated; last occurrence wins)
  - `W_SEM_UNSATISFIABLE_VALUE_DOMAIN` (`@enum` combined with `@len`/`@range`/`@regex` yields an empty effective domain)
  - `W_SEM_RECOVER_BUDGET_WITHOUT_RECOVER` (`@recover_budget` present while `@recover` is not enabled)
  - `W_SEM_RECOVER_PARSE_BUDGET_WITHOUT_RECOVER` (`@recover_parse_budget` present while `@recover` is not enabled)
  - `W_SEM_RECOVER_GLOBAL_BUDGET_WITHOUT_RECOVER` (`@recover_global_budget` present while `@recover` is not enabled)
  - `W_SEM_RECOVERY_HINT_WITHOUT_RECOVER` (`@sync`/`@panic_until` present while `@recover` is not enabled)
  - `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT` (`@requires`/`@implies` present while `@constraint` is missing)
  - `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET` (`@critical_path` enabled while effective `@coverage_target` is missing/zero)
  - `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE` (`@negative` enabled while `@invalid_case` is missing/disabled)
  - `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP` (`@seed_group` present while `@deterministic_group` is missing/disabled)
  - `W_SEM_TOKEN_STEERING_PRECEDENCE` (`@pattern/@charset/@token_class` overlap present; deterministic precedence contract applies)
  - `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM` (token steering hints present but rule has no regex atom)
- Grammar ambiguity diagnostics (grammar-aware validation pass):
  - `W_GRAM_AMBIGUOUS_PREFIX` (top-level alternation branches share the same leading quoted terminal; parse selection may depend on branch order)
  - `W_GRAM_FIRST_SET_OVERLAP` (top-level alternation branches have overlapping computed FIRST terminals, including overlaps introduced via nullable prefixes and rule references)
  - `W_GRAM_NULLABLE_BRANCH_SHADOW` (an earlier top-level branch is nullable and may shadow later alternatives in ordered choice)

### 8.8 Practical Examples

#### Example: Numeric literal rule with transform + range
```ebnf
number = regex("[0-9]+") @transform: str::parse::<i64>().unwrap_or(0) @range: 0..255 ;
```
- Parser: parses token, enforces `0..255`, then transforms to typed terminal string output.
- Stimuli: prefers values in range domain and still validates against regex.

#### Example: Identifier rule with enum + regex + len
```ebnf
ident = regex("[A-Z]+") @enum: ["AA", "AB", "BC"] @regex: "^A[A-Z]$" @len: [2, 2] ;
```
- Effective allowed values are intersection of all constraints (`AA`, `AB`).

#### Example: Ambiguous alternation prefix warning
```ebnf
statement = "if" expr | "if" stmt ;
```
- Grammar-aware validator emits `W_GRAM_AMBIGUOUS_PREFIX` for `statement` because both branches start with the same quoted terminal (`"if"`).

#### Example: FIRST-set overlap via nullable prefix
```ebnf
prefix = "a"? ;
statement = prefix "if" | "if" expr ;
```
- Grammar-aware validator emits `W_GRAM_FIRST_SET_OVERLAP` because `prefix "if"` can start with `"if"` when `prefix` is empty.

#### Example: Nullable branch shadow risk
```ebnf
statement = "if"? | "while" expr ;
```
- Grammar-aware validator emits `W_GRAM_NULLABLE_BRANCH_SHADOW` because branch 1 is nullable and appears before later alternatives.

#### Example: Expression rule branch steering
```ebnf
expr = term | expr "+" term ;
@precedence: [1, 9]
@associativity: right
```
- Parser and stimuli both bias toward later branch under exact ties because of right associativity and higher branch precedence value.

#### Example: Explicit branch policy steering
```ebnf
stmt = if_stmt | while_stmt ;
@branch_policy: priority_first
@priority: [1, 10]
```
- Parser and stimuli apply branch policy before fallback tie-breaks:
  - `longest_match` (default),
  - `ordered`,
  - `priority_first`.

#### Example: Coverage-target steering baseline
```ebnf
expr = additive | multiplicative ;
@coverage_target: 3
@critical_path: true
```
- Current behavior:
  - validator enforces typed payloads and coherence warnings,
  - stimuli branch exploration and gap-report priority ordering are boosted for this rule,
  - generated parser records SC-10 events/counters for this rule and selected branches.

#### Example: Relational contract baseline (validator + parser runtime)
```ebnf
pair = ident ":" ident ;
@constraint: "$1 != $2"
@requires: ["$1", "$2"]
@implies: "$1 => $2"
```
- Current behavior:
  - validator enforces payload shapes/coherence,
  - generated parser enforces relational contract checks at runtime,
  - stimuli generator enforces relational checks during root sequence synthesis retries.

#### Example: Relational coherence warning
```ebnf
pair = ident ":" ident ;
@requires: ["$1", "$2"]
```
- Validator emits:
  - `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`
  - because `@requires` is present without a base `@constraint`.

#### Example: Recovery hints runtime baseline
```ebnf
stmt = declaration | assignment ;
@recover: true
@sync: [";", "end"]
@panic_until: ["}"]
```
- Validator enforces typed payloads and contract coherence.
- Current generated parser behavior:
  - if all OR branches fail and effective `@recover` is truthy, runtime recovery is attempted,
  - optional scoped budgets can limit successful recoveries:
    - `@recover_budget` (per rule per parse),
    - `@recover_parse_budget` (all rules in one parse),
    - `@recover_global_budget` (parser lifetime),
  - parser scans from rule start for nearest configured marker token,
  - token precedence on same location is deterministic: `panic_until` over `sync`,
  - parser advances past the chosen marker, or falls back to EOF skip when no marker is found,
  - unrecoverable no-progress cases still return normal backtrack errors,
  - structured recovery events are recorded and exposed through generated parser APIs:
    - `recovery_events()`
    - `take_recovery_events()`
    - `recovery_event_count()`
    - `recovery_parse_count()`
    - `recovery_global_count()`
  - event shape includes:
    - `rule_name`
    - `parse_start`
    - `previous_position`
    - `new_position`
    - `marker_kind` (`panic_until`, `sync`, `eof_fallback`)
    - optional marker metadata (`marker_position`, `marker_value`).
- Stimuli behavior in this stage:
  - when all OR branches fail and effective `@recover` is truthy, fallback samples can be emitted from recovery markers,
  - marker preference is deterministic: first non-empty `@panic_until` token, then first non-empty `@sync` token,
  - dedicated recovery-focused negative-case generation modes are available:
    - `baseline`
    - `recovery_biased`
    - `near_sync_negative`

### 8.9 Gate/Test Coverage for Semantic Steering
- `make -C rust semantic_usage_gate`
- Included in:
  - `make -C rust annotation_contract_gate`
- Coverage includes parser + stimuli semantic usage tests (`semantic_usage_*`), including value-domain steering, directive routing, parser recovery-hook codegen, and stimuli recovery-fallback regressions.
- Coverage also includes SC-04 steering precedence/runtime tests (`token_class`, `charset`, `pattern`) across parser and stimuli paths.

### 8.10 Steering Roadmap References
- `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

### 8.11 Built-In vs Annotation Policy
- Keep built-in behavior minimal and invariant-focused:
  - correctness
  - safety bounds
  - deterministic contracts
- Express domain/project semantics via typed annotations where possible.
- Precedence order:
  1. built-in correctness/safety
  2. supported semantic directives
  3. fallback defaults

Return-annotation completeness reference (non-negotiable):
- `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` ("Return Annotation No-Compromise Contract")
- Return annotations remain critical-path AST-shaping behavior with no intentional feature compromise.

### 8.12 SC-07 Recovery Deep-Dive (Parser + Stimuli)

This section focuses only on:
- `@recover`
- `@recover_budget`
- `@recover_parse_budget`
- `@recover_global_budget`
- `@sync`
- `@panic_until`

#### 8.12.1 Directive Payload Forms

Valid `@recover` payloads (typed bool):
```ebnf
@recover: true
@recover: false
@recover: "yes"
@recover: 1
```

Valid `@recover_budget` payloads (typed non-negative integer):
```ebnf
@recover_budget: 0
@recover_budget: 2
@recover_budget: "5"
```

Valid `@recover_parse_budget` payloads (typed non-negative integer):
```ebnf
@recover_parse_budget: 0
@recover_parse_budget: 4
@recover_parse_budget: "8"
```

Valid `@recover_global_budget` payloads (typed non-negative integer):
```ebnf
@recover_global_budget: 0
@recover_global_budget: 16
@recover_global_budget: "32"
```

Valid `@sync/@panic_until` payloads:
```ebnf
@sync: ";"
@sync: [";", "end", "\n"]
@panic_until: "}"
@panic_until: ["}", "endmodule"]
```

Invalid payload examples (validator warnings):
```ebnf
@recover: maybe              # W_SEM_INVALID_RECOVER_PAYLOAD
@recover_budget: -1          # W_SEM_INVALID_RECOVER_BUDGET_PAYLOAD
@recover_budget: many        # W_SEM_INVALID_RECOVER_BUDGET_PAYLOAD
@recover_parse_budget: -2    # W_SEM_INVALID_RECOVER_PARSE_BUDGET_PAYLOAD
@recover_global_budget: bad  # W_SEM_INVALID_RECOVER_GLOBAL_BUDGET_PAYLOAD
@sync: []                    # W_SEM_INVALID_SYNC_PAYLOAD
@panic_until: []             # W_SEM_INVALID_PANIC_UNTIL_PAYLOAD
@sync: [";"]                 # plus @recover missing/false => W_SEM_RECOVERY_HINT_WITHOUT_RECOVER
@recover_budget: 3           # plus @recover missing/false => W_SEM_RECOVER_BUDGET_WITHOUT_RECOVER
@recover_parse_budget: 3     # plus @recover missing/false => W_SEM_RECOVER_PARSE_BUDGET_WITHOUT_RECOVER
@recover_global_budget: 3    # plus @recover missing/false => W_SEM_RECOVER_GLOBAL_BUDGET_WITHOUT_RECOVER
```

#### 8.12.2 Parser Runtime Behavior Examples

Example A: `@recover` disabled
```ebnf
stmt = declaration | assignment ;
@recover: false
@sync: [";"]
```
- OR-branch failure returns normal backtrack error.
- Recovery hints are inactive.

Example B: token-based recovery with `panic_until` priority
```ebnf
stmt = declaration | assignment ;
@recover: true
@recover_budget: 2
@recover_parse_budget: 6
@recover_global_budget: 20
@sync: [";"]
@panic_until: ["}"]
```
- On full OR failure:
  - parser scans from rule start,
  - nearest marker wins,
  - tie on same position: `panic_until` beats `sync`,
  - parser advances past chosen marker and continues,
  - recovery succeeds only while all active budgets still have capacity:
    - rule-local `@recover_budget`,
    - parse-scope `@recover_parse_budget`,
    - parser-lifetime `@recover_global_budget`.

Example C: no marker found
```ebnf
stmt = declaration | assignment ;
@recover: true
@sync: [";"]
@panic_until: ["}"]
```
- If none of the markers are found in remaining input:
  - parser falls back to EOF advance.

Example D: no forward progress possible
- If recovery cannot advance parser position, parse returns backtrack error (no silent success).

Example E: budget exhaustion blocks further recovery
```ebnf
stmt = declaration | assignment ;
@recover: true
@recover_budget: 1
@sync: [";"]
```
- First eligible failure can recover.
- Subsequent eligible failures in the same parse for `stmt` do not recover once budget is exhausted.

Example F: parse-scope budget exhaustion
```ebnf
stmt = declaration | assignment ;
@recover: true
@recover_parse_budget: 2
@sync: [";"]
```
- After two successful recoveries (across all rules) in one parse call, further recoveries are blocked for that parse run.

Example G: global budget exhaustion
```ebnf
stmt = declaration | assignment ;
@recover: true
@recover_global_budget: 5
@sync: [";"]
```
- After five successful recoveries on the parser instance lifetime, additional parse calls do not recover until a new parser instance is created.

#### 8.12.3 Structured Recovery Event API (Generated Parser)

Generated parsers expose:
- `recovery_events()`
- `take_recovery_events()`
- `recovery_event_count()`
- `recovery_parse_count()`
- `recovery_global_count()`

Event model:
- `RecoveryEvent`
  - `rule_name`
  - `parse_start`
  - `previous_position`
  - `new_position`
  - `marker_kind` (`PanicUntil`, `Sync`, `EofFallback`)
  - `marker_position` (`Option<usize>`)
  - `marker_value` (`Option<String>`)

Minimal usage pattern:
```rust
let logger = Box::new(crate::NoOpLogger);
let mut parser = GeneratedParser::new(input, logger);
let result = parser.parse();

for ev in parser.recovery_events() {
    println!(
        "rule={} kind={:?} {}->{} marker={:?}",
        ev.rule_name, ev.marker_kind, ev.previous_position, ev.new_position, ev.marker_value
    );
}
```

Notes:
- `parse()` clears prior recovery events at parse start.
- `parse_full()` delegates through `parse()` and uses the same event lifecycle.
- `take_recovery_events()` drains current events for one-shot consumers.

#### 8.12.4 Stimuli Behavior Examples

Example H: recovery fallback sample (panic marker preferred)
```ebnf
start = missing_left | missing_right ;
@recover: true
@sync: [";"]
@panic_until: ["}"]
```
- If all OR branches fail during stimuli generation:
  - emitted fallback sample is `"}"` (first non-empty `@panic_until` token).

Example I: sync fallback sample
```ebnf
start = missing_left | missing_right ;
@recover: true
@sync: [";"]
```
- With no `@panic_until`, fallback sample becomes first non-empty `@sync` token (`";"` here).

Example J: fallback inactive without `@recover`
```ebnf
start = missing_left | missing_right ;
@sync: [";"]
```
- OR exhaustion remains an error; no recovery fallback sample is emitted.

#### 8.12.5 Dedicated Stimuli Modes (`--recovery-stimuli-mode`)

Supported values:
- `baseline` (default)
  - standard generation behavior.
- `recovery_biased`
  - for recover-enabled entry rules, generate a base sample then inject recovery marker context.
  - if base generation fails but recovery marker exists, marker-only sample is emitted.
- `near_sync_negative`
  - for recover-enabled entry rules, generate near-sync negative-case sample by injecting deterministic invalid noise adjacent to selected marker.
  - without recover-enabled contract, mode falls back to baseline behavior.

Example K: recovery-biased mode
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 20 \
  --recovery-stimuli-mode recovery_biased
```

Example L: near-sync negative mode
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- \
  generated/semantic_annotation.json \
  --generate-stimuli \
  --count 20 \
  --recovery-stimuli-mode near_sync_negative
```

#### 8.12.6 Determinism + Precedence Summary

Parser determinism:
1. earliest marker location wins
2. same location: `panic_until` before `sync`
3. no marker: EOF fallback
4. recovery is blocked if any active budget is exhausted:
   `@recover_budget`, `@recover_parse_budget`, `@recover_global_budget`

Stimuli determinism:
1. first non-empty `@panic_until` token
2. else first non-empty `@sync` token
3. only active when effective `@recover` is true

#### 8.12.7 Recommended Authoring Patterns

Pattern 1:
- keep `@recover` explicit (`true`/`false`), do not rely on ambiguous raw text

Pattern 2:
- set both `@panic_until` and `@sync`:
  - `@panic_until` for hard stop markers (`"}"`, `"endmodule"`)
  - `@sync` for statement separators (`";"`, `"end"`)

Pattern 3:
- add at least one semantic usage regression test whenever recovery directives are changed

Pattern 4:
- use all three budget scopes deliberately:
  - `@recover_budget` to cap local rule churn,
  - `@recover_parse_budget` to bound one parse run,
  - `@recover_global_budget` to bound long-lived parser instances.

### 8.13 SC-09 Relational Constraint Contract (Validator + Parser + Stimuli Baseline)

This section focuses on:
- `@constraint`
- `@requires`
- `@implies`

Current stage:
- typed validator contract is active,
- parser runtime relational steering is active,
- stimuli runtime relational steering baseline is active for root sequence synthesis.

#### 8.13.1 Payload Forms

Valid examples:
```ebnf
@constraint: "$1 != $2"
@constraint: lhs.id == rhs.id
@requires: ["$1", "$2", "lhs.id"]
@requires: rhs
@implies: "$1 => $2"
@implies: lhs.ready => rhs.valid
```

Invalid examples:
```ebnf
@constraint: ""              # W_SEM_INVALID_CONSTRAINT_PAYLOAD
@requires: ["1bad"]          # W_SEM_INVALID_REQUIRES_PAYLOAD
@implies: "lhs -> rhs"       # W_SEM_INVALID_IMPLIES_PAYLOAD
@implies: "lhs => rhs => z"  # W_SEM_INVALID_IMPLIES_PAYLOAD
```

#### 8.13.2 Coherence Rule

If `@requires` or `@implies` is present without `@constraint`, validator emits:
- `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`

Example:
```ebnf
pair = ident ":" ident ;
@requires: ["$1", "$2"]
```

#### 8.13.3 Deterministic Contract Notes

- Same-directive duplicates are still last-wins (`W_SEM_DIRECTIVE_OVERRIDDEN`).
- Strict mode warning-class promotion is policy-controlled via `PGEN_STRICT_SEMANTIC_WARNING_CODES`.
- Runtime parser enforcement activates only when `@constraint` is present:
  - `@requires` references must resolve and be non-empty.
  - `@constraint` expression is evaluated against capture/reference values.
  - `@implies` is enforced as antecedent truth implies consequent truth.
- Runtime stimuli enforcement activates only when `@constraint` is present on root-sequence rules:
  - generation retries until relational checks pass (or attempt budget is exhausted),
  - same `@requires/@constraint/@implies` checks gate sample acceptance.
- Reference resolution supports positional (`$1`, `$2.field`) and named dotted references (`lhs.id`) including `.len` suffix (for example `$1.len >= 1`).
- Stimuli reference support includes nested named/positional paths (for example `lhs.id`, `$1.id`, `$3.id.len`) when referenced capture values are:
  - structured JSON-like payloads,
  - non-structured object-like payloads parsed from `=/:` key-value pairs (with `,`/`;`/newline delimiters and optional outer wrappers).
- On retry exhaustion, stimuli returns structured diagnostics including:
  - `relational_failures=<N>`
  - `generation_failures=<N>`
  - `top_violations=[<count>x <reason> | ...]` (ranked top causes)
  - `likely_unsatisfiable=<bool>` (true when one cause consistently explains all relational failures)
- These directives now provide parse+validate plus parser+stimuli runtime contract surface (`Tier 3` baseline).

#### 8.13.4 Nested-Path Stimuli Example

```ebnf
pair = lhs "|" rhs ;
lhs = "{\"id\":\"" ident "\"}" ;
rhs = "{\"id\":\"" ident "\"}" ;
ident = regex("^[A-Z]{2}$") @enum: ["AA", "BB"] ;

@constraint: "lhs.id != rhs.id && $1.id.len == 2"
@requires: ["lhs.id", "rhs.id", "$1.id.len"]
```

Expected behavior:
- parser and stimuli both resolve nested paths (`lhs.id`, `$1.id`) and optional `.len`,
- stimuli retries sequence synthesis until constraint holds,
- emitted samples satisfy the contract (for example `{"id":"AA"}|{"id":"BB"}`).

### 8.14 SC-10 Coverage-Target Steering Contract (Validator + Parser + Stimuli Baseline)

This section focuses on:
- `@coverage_target`
- `@critical_path`

Current stage:
- typed validator contract is active,
- stimuli coverage/gap steering baseline is active,
- parser instrumentation baseline from SC-10 hints is active.
- strict warning policy default promotes malformed SC-10 payload diagnostics to errors when strict validation is enabled.

#### 8.14.1 Payload Forms

Valid examples:
```ebnf
@coverage_target: 2
@coverage_target: true
@coverage_target: false
@critical_path: true
@critical_path: false
```

Invalid examples:
```ebnf
@coverage_target: "boost"      # W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD
@critical_path: "urgent"       # W_SEM_INVALID_CRITICAL_PATH_PAYLOAD
```

#### 8.14.2 Coherence Rule

If `@critical_path` is enabled while effective `@coverage_target` is missing or zero, validator emits:
- `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`

Example:
```ebnf
expr = atom ;
@critical_path: true
```

#### 8.14.3 Stimuli Steering Behavior

- SC-10 hints participate in OR branch guidance multiplier calculation.
- Branches in tagged rules and branches referencing tagged rules receive higher exploration pressure.
- Gap-report rule/branch priority scores include SC-10 semantic bonuses.
- This affects target ordering for gap-driven injection without changing grammar semantics.

#### 8.14.4 Parser Instrumentation Behavior

- Successful parses on effective SC-10 rules emit `CoverageTargetEvent` with:
  - `rule_name`
  - `parse_start`
  - `parse_end`
  - `branch_index` (for OR rules)
  - `coverage_target_weight`
  - `critical_path`
- Accessors available on generated parsers:
  - `coverage_target_events()`
  - `take_coverage_target_events()`
  - `coverage_target_event_count()`
  - `coverage_target_rule_hits()`
  - `coverage_target_branch_hits()`
- Instrumentation remains inactive when effective `@coverage_target` weight is zero.

### 8.15 SC-11 Negative-Case Steering Contract (Validator + Parser + Stimuli Baseline)

This section focuses on:
- `@invalid_case`
- `@negative`

Current stage:
- typed validator contract is active,
- parser expected-failure event baseline is active,
- stimuli invalid/near-invalid mutation baseline is active.

#### 8.15.1 Payload Forms

Valid examples:
```ebnf
@invalid_case: true
@invalid_case: false
@negative: true
@negative: false
```

Invalid examples:
```ebnf
@invalid_case: "maybe"      # W_SEM_INVALID_INVALID_CASE_PAYLOAD
@negative: "sometimes"      # W_SEM_INVALID_NEGATIVE_PAYLOAD
```

#### 8.15.2 Coherence Rule

If `@negative` is enabled while effective `@invalid_case` is missing or disabled, validator emits:
- `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE`

Example:
```ebnf
stmt = atom ;
@negative: true
```

#### 8.15.3 Parser Runtime Behavior

- On rule failure with effective `@invalid_case: true`, generated parser records `NegativeCaseEvent`:
  - `rule_name`
  - `parse_start`
  - `failure_position`
  - `negative`
  - `error_kind`
- Accessors available on generated parsers:
  - `negative_case_events()`
  - `take_negative_case_events()`
  - `negative_case_event_count()`
  - `negative_case_rule_hits()`

#### 8.15.4 Stimuli Runtime Behavior

- With effective `@invalid_case: true`, entry stimuli are deterministically mutated toward invalid/near-invalid shape.
- With both `@invalid_case: true` and `@negative: true`, a deterministic negative-case marker suffix is appended.
- With `@negative: true` only, steering remains inactive (coherence rule above).

### 8.16 SC-12 Determinism Partition Contract (Validator + Parser + Stimuli Baseline)

This section focuses on:
- `@seed_group`
- `@deterministic_group`

Current stage:
- typed validator contract is active,
- parser deterministic partition steering baseline is active,
- stimuli deterministic seed partition routing baseline is active.

#### 8.16.1 Payload Forms

Valid examples:
```ebnf
@seed_group: "stable.expr"
@deterministic_group: true
@deterministic_group: false
@deterministic_group: "stable.expr"
```

Invalid examples:
```ebnf
@seed_group: "group with spaces"    # W_SEM_INVALID_SEED_GROUP_PAYLOAD
@deterministic_group: "%%%"         # W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD
```

#### 8.16.2 Coherence Rule

If `@seed_group` is present while effective `@deterministic_group` is missing or false, validator emits:
- `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP`

#### 8.16.3 Parser Deterministic Partition Behavior

- For OR rules under `@branch_policy: ordered`, effective deterministic partition hints apply a stable branch-evaluation offset before first-success short-circuit.
- Offset is deterministic per group key + branch count (`hash(group_key) % branch_count`).
- Group key resolution order:
  - explicit `@seed_group`,
  - else explicit group label embedded in `@deterministic_group`,
  - else fallback `rule.<rule_name>`.
- Generated parser exposes runtime override controls for embedders:
  - enum: `DeterministicPartitionRuntimeMode`
    - `AnnotationDriven` (default)
    - `ForceEnabled`
    - `ForceDisabled`
  - accessors:
    - `deterministic_partition_runtime_mode()`
    - `set_deterministic_partition_runtime_mode(...)`
- Runtime mode controls effective partition behavior for ordered OR branch ordering and partition telemetry:
  - `AnnotationDriven`: follow annotation payload,
  - `ForceEnabled`: enable partition behavior regardless of annotation,
  - `ForceDisabled`: disable partition behavior regardless of annotation.
- Generated parser emits typed partition telemetry on effective deterministic-group rules:
  - event type: `DeterministicPartitionEvent { rule_name, parse_start, parse_end, group_key }`
  - accessors:
    - `deterministic_partition_events()`
    - `take_deterministic_partition_events()`
    - `deterministic_partition_event_count()`
    - `deterministic_partition_rule_hits()`

#### 8.16.4 Stimuli Deterministic Partition Behavior

- When effective `@deterministic_group` is enabled and `--seed` is provided:
  - generator derives deterministic partition seeds per semantic group,
  - per-group counters produce stable sequences,
  - interleaving calls across different groups does not perturb each group's sequence.
- Group key resolution order:
  - explicit `@seed_group`,
  - else explicit group label embedded in `@deterministic_group`,
  - else fallback `rule.<entry_rule>`.
- When `@deterministic_group` is disabled, `@seed_group` has no runtime effect.

### 8.17 SC-04 Token-Family Steering Deep-Dive (`@token_class`, `@charset`, `@pattern`)

This section explains the exact purpose of the three SC-04 directives and when to use each one.

#### 8.17.1 Why three directives exist

- `@token_class`:
  - high-level intent-based steering.
  - use when you want a standard lexical family and do not care about exact regex text.
  - examples: identifier, integer, float, boolean.
- `@charset`:
  - medium-level steering for allowed character families.
  - use when you want a constrained class without writing a full regex.
  - examples: uppercase hex-like symbols, limited symbol alphabet.
- `@pattern`:
  - full explicit regex steering.
  - use when you need exact lexical acceptance behavior.

In short:
1. `@token_class` is semantic intent.
2. `@charset` is character-domain intent.
3. `@pattern` is exact matcher intent.

#### 8.17.2 Deterministic precedence contract

When multiple SC-04 directives are present on the same rule, effective steering is deterministic:
1. `@pattern` wins.
2. else `@charset` wins.
3. else `@token_class` wins.

Validator emits `W_SEM_TOKEN_STEERING_PRECEDENCE` to make overlap visible.

#### 8.17.3 Parser and stimuli use the same effective policy

- Parser:
  - generated regex matching for regex atoms uses the effective SC-04 steering regex.
- Stimuli:
  - regex sample generation also uses the same effective SC-04 steering regex.

This avoids split behavior where parser and stimuli interpret SC-04 differently.

#### 8.17.4 Inactive-steering condition

SC-04 steering requires at least one regex atom in the rule.

If a rule has no regex atom, validator emits:
- `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`

This is intentional so authors can detect directives that are syntactically valid but operationally inactive.

#### 8.17.5 Practical authoring patterns

Pattern A: coarse lexical class
```ebnf
ident = regex("[0-9]+") @token_class: identifier ;
```
- Effective behavior uses identifier family matching/generation.

Pattern B: constrained alphabet
```ebnf
hex_chunk = regex("[a-z]+") @charset: [A-F0-9] ;
```
- Effective behavior uses `[A-F0-9]+`.

Pattern C: exact override
```ebnf
flag = regex("[a-z]+")
  @token_class: identifier
  @charset: [A-F0-9]
  @pattern: ^Q{2}$ ;
```
- Effective behavior uses `^Q{2}$` due to precedence.

#### 8.17.6 Recommended usage guideline

1. Start with `@token_class` when onboarding a grammar quickly.
2. Move to `@charset` when class-level control is needed.
3. Use `@pattern` for final precise behavior.
4. Keep only one SC-04 directive when possible to reduce ambiguity/noise.

#### 8.17.7 Tier-4 Contract Gate (SC-04)

SC-04 is now gate-enforced with an explicit contract slice and differential taxonomy checks.

Contract corpus:
- `rust/test_data/semantic_annotation/sc04_contract.json`

Gate commands:
```bash
make -C rust sc04_contract_gate
```

Included in the main annotation contract gate:
```bash
make -C rust annotation_contract_gate
```

What `sc04_contract_gate` enforces:
1. Typed SC-04 payload/coherence validator tests (`@token_class/@charset/@pattern` + precedence/inactive-steering diagnostics).
2. Parser/stimuli semantic-usage SC-04 runtime steering tests.
3. Bootstrap and generated semantic parser round-trip contract suite for SC-04.
4. Differential mismatch taxonomy parity check on the SC-04 contract slice:
- only known taxonomy categories are accepted,
- category-count totals must match `mismatched_cases`,
- current SC-04 parity expectation is `mismatched_cases == 0`.

## 9) Differential Testing and Drift Management

Reference help:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- --help
```

Key differential modes:

1. Raw differential run:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- \
  --differential --parser semantic --differential-report-json /tmp/semantic_diff.json
```

2. Comparable-only parity run (expectation-aligned corpus only):
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- \
  --differential --parser return --differential-comparable-only \
  --differential-report-json /tmp/return_parity_diff.json

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin test_runner -- \
  --differential --parser semantic --differential-comparable-only \
  --differential-report-json /tmp/semantic_parity_diff.json
```

3. Refresh tracked mismatch baseline:
```bash
make -C rust SHELL=/bin/bash differential_refresh_baseline
```

4. Regression-only gate (fail only on NEW mismatches):
```bash
make -C rust SHELL=/bin/bash differential_regression_gate
```

5. Return parity gate (comparable corpus must be zero mismatch):
```bash
make -C rust SHELL=/bin/bash return_parity_gate
```

Tracked baselines:
- `rust/test_data/differential_baseline/return_annotation_baseline.json`
- `rust/test_data/differential_baseline/semantic_annotation_baseline.json`

## 10) CI Gates and What They Protect

- `fixed-point-gate`
  - deterministic bootstrap artifact regeneration
- `sota_exit_gate` (local aggregate target)
  - one-shot release-grade aggregate check for fixed-point, annotation, non-annotation EBNF quality loop, stimuli-module parity, differential, performance, embedding, and EBNF readiness reporting
  - `rust/target/sota_exit_gate/summary.txt` now surfaces parser-backed quality telemetry from aggregate-scoped EBNF/SV/VHDL closed-loop stages, not only top-level pass/fail rows
- `sota_release_policy` (local utility target)
  - prints the tracked machine policy consumed by `sota_exit_gate`
- `branch_protection_contract_gate` (local gate target)
  - validates the tracked minimum pre-merge required-check contract in `rust/config/branch_protection_policy.json`,
  - fails if a required check disappears, loses workflow coverage, or stops running on `pull_request`
- `sv_declared_shadow_promotion_gate` (local gate target)
  - executes deterministic strict declared-shadow trial matrix and emits:
    - `rust/target/sv_declared_shadow_promotion_gate/work/systemverilog_declared_identifier_promotion_report.json`
  - recommendation:
    - `enable_runtime_declared_identifiers` or `hold`
- `annotation_contract_gate` (local gate target)
  - aggregate annotation contract gate:
    - bootstrap validator + built-in return/semantic suites
    - shared bootstrap/generated return/semantic suites
    - SC Tier-4 semantic contract slices
    - aggregate semantic + return contract gates
    - annotation robustness + closed-loop annotation stimuli quality gates
- `annotation_shared_contract_gate` (local gate target)
  - shared bootstrap/generated annotation contract gate:
    - return shared suite in both parser modes
    - semantic shared suite in both parser modes
- `semantic_usage_gate` (local gate target)
  - focused parser/stimuli semantic leverage contract coverage (`semantic_usage_*`)
- `semantic_runtime_contract_gate` (local gate target)
  - focused semantic runtime contract checks:
    - validator/runtime unit coverage
    - generated semantic parse-tree -> typed-AST conversion regression checks
    - semantic usage gate
- `semantic_ast_roundtrip_gate` (local gate target)
  - focused semantic shared-suite round-trip contract checks in bootstrap and generated modes
- `semantic_full_contract_gate` (local gate target)
  - aggregate semantic contract gate:
    - `semantic_runtime_contract_gate`
    - `semantic_ast_roundtrip_gate`
    - `semantic_differential_regression_gate`
- `annotation_nonbootstrap_e2e_gate` (local gate target)
  - generated-parser end-to-end verification for non-bootstrap annotation flow:
    - non-bootstrap parser generation for return / semantic / regex,
    - parser-backed stimuli generation for return / semantic with structured parseability reports,
    - summary CSV/text now surfaces attempts, accepted/rejected totals, acceptance rate, and report paths for parser-backed rows
- `sc06_contract_gate` (local gate target)
  - SC-06 branch weighting/selection Tier-4 contract:
    - typed branch-policy payload validation contracts,
    - parser/stimuli branch-selection runtime tests,
    - weighted-probability determinism/fallback tests,
    - SC-06 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc05_contract_gate` (local gate target)
  - SC-05 precedence/associativity Tier-4 contract:
    - typed SC-05 payload/coherence validator contracts,
    - parser precedence/associativity runtime contract checks,
    - stimuli precedence/associativity steering contract checks,
    - SC-05 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc08_contract_gate` (local gate target)
  - SC-08 value-domain Tier-4 contract:
    - typed SC-08 payload/coherence validator contracts,
    - parser value-domain runtime contract checks,
    - stimuli value-domain steering contract checks,
    - SC-08 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc07_contract_gate` (local gate target)
  - SC-07 recovery/sync Tier-4 contract:
    - typed recovery payload/coherence validator contracts,
    - parser runtime recovery hook + structured telemetry contract checks,
    - stimuli recovery fallback and recovery-focused mode contract checks,
    - SC-07 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc09_contract_gate` (local gate target)
  - SC-09 relational-constraint Tier-4 contract:
    - typed relational payload/coherence validator contracts,
    - parser relational policy/runtime guard contract checks,
    - stimuli relational synthesis + unsatisfiable diagnostics contract checks,
    - SC-09 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc10_contract_gate` (local gate target)
  - SC-10 coverage-target Tier-4 contract:
    - typed SC-10 payload/coherence validator contracts,
    - parser runtime coverage-target instrumentation contract checks,
    - stimuli coverage steering contract checks,
    - SC-10 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc11_contract_gate` (local gate target)
  - SC-11 negative-case Tier-4 contract:
    - typed SC-11 payload/coherence validator contracts,
    - parser runtime negative-case instrumentation contract checks,
    - stimuli invalid/negative generation contract checks,
    - SC-11 bootstrap/generated contract slice + differential taxonomy parity checks
- `sc12_contract_gate` (local gate target)
  - SC-12 deterministic-partition Tier-4 contract:
    - typed SC-12 payload/coherence validator contracts,
    - parser deterministic-partition runtime contract checks,
    - stimuli deterministic-partition generation contract checks,
    - SC-12 bootstrap/generated contract slice + differential taxonomy parity checks
- `annotation_robustness_gate` (local gate target)
  - advanced return/semantic suites in bootstrap/generated modes + generated parseability/coverage/gap checks
  - parser-backed stimuli rows now emit structured parseability reports and summary CSV/text with attempts, accepted/rejected totals, acceptance rate, and report paths
- `annotation_stimuli_quality_gate` (local gate target)
  - strict deterministic closed-loop verification for return/semantic annotation grammars:
    - baseline parseability/coverage/gap,
    - gap-priority generation with merged-coverage invariants,
    - target-driven generation summary integrity,
    - final gap no-regression checks
  - parser-backed stages now emit aggregated parseability summary artifacts:
    - `summary.csv` / `summary.txt`
    - per-grammar `<label>_parseability_report.json`
    - parseability attempts / accepted / rejected totals, rejection breakdown, acceptance rate, and target-closure context
- `ebnf_stimuli_quality_gate` (local gate target)
  - strict deterministic closed-loop verification for tracked non-annotation EBNFs (separate from annotation loop):
    - `EBNF -> JSON` frontend success (`ebnf_to_json.pl`),
    - parser generation success,
    - baseline/gap-priority/target-driven/final-gap no-regression checks,
    - contract-driven grammar roster from `rust/test_data/grammar_quality/ebnf_stimuli_contract.json`
- `sv_preprocessor_quality_gate` (local gate target)
  - deterministic closed-loop verification for `systemverilog_preprocessor` grammar:
    - stage0 deterministic replay (same seed, same outputs),
    - baseline/gap-priority/target-driven/final-gap no-regression checks,
    - preprocessor-specific coverage assertions (`include/define/conditional/macro` families),
    - deterministic coverage-guided fuzz replay parity checks
  - parser-backed preprocessor stages now emit aggregate parseability telemetry when the generated adapter is available:
    - `parseability_attempts_total`
    - `parseability_accepted_total`
    - `parseability_rejected_total`
    - `parseability_acceptance_rate_percent`
    - `parseability_report_json`
  - parseability mode:
    - `auto` (default): enable parseability/shrink checks when parser-registry adapter exists,
    - `1`: require parseability adapter (fail if unavailable),
    - `0`: coverage/gap-only mode.
- `stimuli_module_parity_gate` (local gate target)
  - strict deterministic parity verification between in-memory stimuli generation and generated `*_stimuli.rs` artifacts:
    - same grammar + seed + generation config,
    - identical sample corpus,
    - identical coverage metrics JSON and gap report JSON (canonicalized compare),
    - for parseability-required grammars, identical parseability report JSON including attempts / accepted / rejected / rejection breakdown,
    - summary CSV/text now surfaces parseability attempts, accepted/rejected totals, acceptance rate, and both report artifact paths,
    - contract-driven grammar roster from `rust/test_data/grammar_quality/stimuli_module_parity_contract.json`
- `ast_dump_contract_gate` (local gate target)
  - deterministic + bounded AST dump contract verification for both dump surfaces:
    - generation-input AST dump replay determinism and truncation envelope checks,
    - parser-returned AST dump replay determinism and truncation envelope checks,
    - negative-path write-failure checks for both dump surfaces.
- `ebnf_frontend_readiness` (local report target)
  - executes `EBNF -> JSON -> parser/stimuli` readiness checks for `ebnf/json/regex` grammars
- `ebnf_frontend_gate` (local strict target)
  - same checks, but fails on any grammar-flow failure
- `ebnf_frontend_dual_run_diff` (local report target)
  - executes Perl-vs-Rust (`generated/ebnf.rs`) frontend differential report for `ebnf/json/regex`
  - report now also includes Rust raw-AST export comparison against Perl `raw_ast` output:
    - `raw_ast_status=parity|perl_under_reports|rust_under_reports|divergent`
    - missing-rule counts on each side
    - per-grammar `raw_ast_compare_json` artifact
- `ebnf_frontend_dual_run_gate` (local strict target)
  - same dual-run differential checks, but fails on unexpected mismatches/failures
  - known legacy-Perl subset under-reporting is surfaced as informational telemetry (`perl_under_reports`) instead of being treated as a hard parity failure by itself
- `performance-gate`
  - throughput/latency/failure thresholds
- `differential-regression-gate`
  - no new generated-vs-bootstrap mismatches
- `return_parity_gate` (local gate target)
  - zero return mismatches on expectation-aligned (comparable) differential corpus
  - generated return round-trip path emits canonical typed output (not parse-only identity echo)
- `semantic_differential_regression_gate` (local gate target)
  - semantic differential regression on expectation-aligned (comparable) corpus only
  - tracks zero baseline debt for comparable semantic cases while allowing explicit bootstrap-only legacy cases to stay outside parity debt accounting
- `return_annotation_support_gate` (local gate target)
  - aggregate return-annotation support proof:
    - parser-registry support audit
    - `return_full_contract_gate`
    - `return_annotation_exhaustiveness_gate`
- `return_runtime_semantics_gate` (local gate target)
  - focused return runtime contract checks:
    - typed return AST bootstrap/runtime checks,
    - generated-parser return parse-tree -> typed-AST conversion regression check,
    - return validator checks
- `return_ast_roundtrip_gate` (local gate target)
  - focused return canonical round-trip contract checks (`test_round_trip_runner` + return shared contract suite)
- `return_full_contract_gate` (local gate target)
  - aggregate return contract gate:
    - `return_runtime_semantics_gate`
    - `return_ast_roundtrip_gate`
    - `return_parity_gate`
- `embedding_api_gate` (local gate target)
  - contract stability for embedding API behavior

Workflow files:
- `.github/workflows/branch-protection-contract-gate.yml`
- `.github/workflows/fixed-point-gate.yml`
- `.github/workflows/performance-gate.yml`
- `.github/workflows/differential-regression-gate.yml`
- `.github/workflows/sota-exit-gate.yml`
- `.github/workflows/ebnf-frontend-dual-run-diff.yml`

EBNF frontend readiness commands:
```bash
make -C rust SHELL=/bin/bash ebnf_frontend_readiness
make -C rust SHELL=/bin/bash ebnf_frontend_gate
```
- Rust frontend path:
```bash
PGEN_EBNF_FRONTEND_IMPL=rust make -C rust SHELL=/bin/bash ebnf_frontend_readiness
PGEN_EBNF_FRONTEND_IMPL=rust make -C rust SHELL=/bin/bash ebnf_frontend_gate
```
- Notes:
  - `PGEN_EBNF_FRONTEND_IMPL=perl` remains the default.
  - the Rust path now handles multiline semantic annotation blocks in tracked grammars such as `grammars/regex.ebnf`.
  - readiness output now distinguishes plain frontend viability from parser-backed validation:
    - `parser_registry_support`
    - `parseability`
    - `parseability_attempts`
    - `parseability_accepted`
    - `parseability_rejected`
    - `parseability_acceptance_rate_percent`
    - `parseability_report_json`
  - currently tracked parser-backed readiness coverage is now available for all tracked grammars:
    - `ebnf`, `json`, and `regex` each rebuild `ast_pipeline` and `parseability_probe` against the freshly generated readiness-run parser for that grammar,
    - the rebuild path is generic and env-driven (`PGEN_EBNF_PARSER_PATH`, `PGEN_JSON_PARSER_PATH`, `PGEN_REGEX_PARSER_PATH`) rather than tied to one grammar,
    - focused evidence at `PGEN_EBNF_FRONTEND_STIMULI_COUNT=2` is currently:
      - `ebnf`: `attempts=4`, `accepted=2`, `rejected=2`, `acceptance_rate_percent=50.00`
      - `json`: `attempts=2`, `accepted=2`, `rejected=0`, `acceptance_rate_percent=100.00`
      - `regex`: `attempts=2`, `accepted=2`, `rejected=0`, `acceptance_rate_percent=100.00`
  - aggregate observability behavior (`sota_exit_gate`):
    - aggregate stage state dir:
      - `rust/target/sota_exit_gate/work/ebnf_frontend_readiness`
    - aggregate output and `summary.txt` include:
      - `ebnf_frontend_readiness_state_dir`
      - `ebnf_frontend_readiness_summary_csv`
      - the full readiness table with parser-backed parseability columns

HDL frontend readiness commands (Pillar 5 kickoff):
```bash
make -C rust SHELL=/bin/bash hdl_frontend_readiness
make -C rust SHELL=/bin/bash hdl_frontend_gate
```
- tracked HDL grammar roster:
  - `grammars/systemverilog.ebnf`
  - `grammars/vhdl.ebnf`
- report mode (`hdl_frontend_readiness`):
  - emits `not_ready` rows when grammar files are missing.
- strict mode (`hdl_frontend_gate`):
  - fails on missing grammar files or failing flow stages.
- readiness stage outputs now include parser replay visibility:
  - `parser_registry_support` (adapter availability),
  - `parseability` (sample replay through generated parser adapter),
  - `parseability_attempts`,
  - `parseability_accepted`,
  - `parseability_rejected`,
  - `parseability_acceptance_rate_percent`,
  - `parseability_report_json`.
- parseability stage behavior:
  - parser-backed readiness now uses the shared `ast_pipeline --validate-parseability --parseability-report-json` path instead of an HDL-only manual retry loop.
  - the gate rebuilds both `ast_pipeline` and `parseability_probe` against the freshly generated HDL parser from the current readiness run before measuring parseability.
  - retry budget env:
    - `PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS` (default `50` requested attempts per emitted sample; the gate maps this into the shared total attempt budget passed to `--parseability-max-attempts`).
- current active HDL grammar status:
  - `grammars/systemverilog.ebnf` is now the active flattened dual-profile SystemVerilog grammar derived from the IEEE 1800-2017/2023 markdown workspaces.
  - `grammars/systemverilog.ebnf` passes `EBNF -> JSON -> parser -> stimuli`.
  - `grammars/vhdl.ebnf` passes `EBNF -> JSON -> parser -> stimuli` (initial seed baseline).
  - strict HDL gate (`make -C rust SHELL=/bin/bash hdl_frontend_gate`) is now green for both tracked grammars.
  - aggregate policy default now promotes HDL readiness to strict required mode (`PGEN_SOTA_POLICY_REQUIRE_HDL_FRONTEND_STRICT=1`).
- aggregate observability behavior (`sota_exit_gate`):
  - aggregate stage state dir:
    - `rust/target/sota_exit_gate/work/hdl_frontend_readiness`
  - aggregate output and `summary.txt` include:
    - `hdl_frontend_readiness_state_dir`
    - `hdl_frontend_readiness_summary_csv`
    - the full readiness table with parser-backed parseability columns
- SystemVerilog syntax-closure tracking artifact:
  - `SV_GRAMMAR_COVERAGE_MATRIX.md`
  - contains Annex-A-aligned anchor mapping, grouped per-rule coverage status, and explicit unresolved-reference closure debt for the current active grammar.
  - unresolved rule-reference debt in `grammars/systemverilog.ebnf` is zero; remaining closure work is parser-quality hardening and higher-level semantic legality.

EBNF frontend dual-run commands:
```bash
make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff
make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate
```

- dual-run report semantics:
  - parser/full-parse parity remains required for all tracked grammars,
  - the report now also compares Perl `raw_ast` rule-name sets against the Rust frontend `raw_ast` export,
  - when the Rust rule-name set is a strict superset of the Perl rule-name set, the row is reported as `raw_ast_status=perl_under_reports` with explicit missing-rule counts and artifact paths instead of being silently treated as plain parity.

Aggregate SOTA policy note:
- dual-run strict mode is required by default in `rust/config/sota_exit_policy.env` (`PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`).

Annotation contract/robustness commands:
```bash
make -C rust SHELL=/bin/bash annotation_robustness_gate
make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate
make -C rust SHELL=/bin/bash sc12_contract_gate
make -C rust SHELL=/bin/bash sc11_contract_gate
make -C rust SHELL=/bin/bash sc10_contract_gate
make -C rust SHELL=/bin/bash sc09_contract_gate
make -C rust SHELL=/bin/bash sc06_contract_gate
make -C rust SHELL=/bin/bash sc05_contract_gate
make -C rust SHELL=/bin/bash sc08_contract_gate
make -C rust SHELL=/bin/bash sc07_contract_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
make -C rust SHELL=/bin/bash return_runtime_semantics_gate
make -C rust SHELL=/bin/bash return_ast_roundtrip_gate
make -C rust SHELL=/bin/bash return_full_contract_gate
make -C rust SHELL=/bin/bash return_parity_gate
```

Non-annotation EBNF closed-loop command:
```bash
make -C rust SHELL=/bin/bash ebnf_stimuli_quality_gate
```
Rust frontend variant:
```bash
PGEN_EBNF_FRONTEND_IMPL=rust PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh
```

SV preprocessor closed-loop command:
```bash
make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
```

SV parser/stimuli preprocess-first closed-loop command:
```bash
make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
```

SV parse-full ratio promotion command:
```bash
make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate
```

VHDL parser/stimuli closed-loop command:
```bash
make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate
```

VHDL strict-promotion trial command:
```bash
make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate
```

SV syntax-closure burn-down no-regression command:
```bash
make -C rust SHELL=/bin/bash sv_syntax_closure_gate
```

Stimuli-module parity command:
```bash
make -C rust SHELL=/bin/bash stimuli_module_parity_gate
```

AST dump contract command:
```bash
make -C rust SHELL=/bin/bash ast_dump_contract_gate
```

Aggregate SOTA gate command:
```bash
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash sota_release_policy
```

Aggregate gate tuning:
- `PGEN_SOTA_RUN_EBNF_READINESS` (`1`/`0`, default `1`)
- `PGEN_SOTA_REQUIRE_EBNF_STRICT` (`1`/`0`, default `0`)
- `PGEN_SOTA_RUN_HDL_FRONTEND_READINESS` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRE_HDL_FRONTEND_STRICT` (`1`/`0`, default from policy file)
- `PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT` (`1`/`0`, default from policy file; current tracked policy default is `1`)
- `PGEN_SOTA_RUN_SV_STIMULI_QUALITY` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRE_SV_STIMULI_QUALITY_STRICT` (`1`/`0`, default from policy file; current tracked policy default is `1`)
- `PGEN_SOTA_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO` (`1`/`0`, default from policy file; controls parse-full ratio strictness passed into `sv_stimuli_quality_gate`)
- `PGEN_SOTA_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO` (`0-100`, default from policy file; minimum parse-full pass ratio percent passed into `sv_stimuli_quality_gate`)
- `PGEN_SOTA_RUN_SV_DECLARED_SHADOW_PROMOTION` (`1`/`0`, default from policy file; controls aggregate execution of declared-shadow promotion trials)
- `PGEN_SOTA_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT` (`1`/`0`, default from policy file; strict mode fails aggregate gate when declared-shadow promotion is not eligible)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TRIALS` (integer `>=1`, default from policy file; aggregate declared-shadow promotion trial count)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_COUNT` (integer `>=1`, default from policy file; per-trial sample count for aggregate declared-shadow promotion)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE` (integer `>=0`, default from policy file; aggregate declared-shadow promotion deterministic seed base)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS` (integer `>=1`, default from policy file; forwarded target-attempt budget for declared-shadow promotion trials)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default from policy file; parse-full mode forwarded to declared-shadow promotion trials)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED` (integer `>=1`, default from policy file; minimum checked shadow samples required by promotion gate)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default from policy file; semantic-closure mode forwarded to declared-shadow promotion trials)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE` (`sv_file`/`sv_parseable_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default from policy file; stimuli mode forwarded to declared-shadow promotion trials)
- `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY` (`0`/`1`, default from policy file; forwarded parseability scope for strict declared-shadow checks inside promotion trials)
- `PGEN_SOTA_RUN_SV_PARSE_FULL_RATIO_PROMOTION` (`1`/`0`, default from policy file; controls aggregate execution of parse-full ratio promotion trials)
- `PGEN_SOTA_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT` (`1`/`0`, default from policy file; strict mode fails aggregate gate when promotion eligibility is not met)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO` (`0-100`, default from policy file; parse-full promotion threshold passed into `sv_parse_full_ratio_promotion_gate`)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS` (integer `>=1`, default from policy file; aggregate promotion trial count)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_COUNT` (integer `>=1`, default from policy file; per-trial sample count for aggregate promotion runs)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE` (integer `>=0`, default from policy file; aggregate promotion deterministic seed base)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE` (integer `>=1`, default from policy file; per-trial seed-base stride for aggregate promotion runs)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default from policy file; parse-full mode forwarded to promotion trials)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default from policy file; semantic-closure mode forwarded to promotion trials)
- `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE` (`sv_file`/`sv_parseable_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default from policy file; stimuli mode forwarded to promotion trials)
- `PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRE_VHDL_STIMULI_QUALITY_STRICT` (`1`/`0`, default from policy file)
- `PGEN_SOTA_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` (integer `>=1`, default from policy file; aggregate replay-budget override forwarded into `vhdl_stimuli_quality_gate` and inherited by `vhdl_strict_promotion_gate`)
- `PGEN_SOTA_RUN_VHDL_STRICT_PROMOTION` (`1`/`0`, default from policy file; controls aggregate execution of VHDL strict-promotion trials)
- `PGEN_SOTA_REQUIRE_VHDL_STRICT_PROMOTION_STRICT` (`1`/`0`, default from policy file; strict mode fails aggregate gate when VHDL strict-promotion eligibility is not met)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_TRIALS` (integer `>=1`, default from policy file; aggregate VHDL strict-promotion trial count)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_COUNT` (integer `>=1`, default from policy file; per-trial sample count for VHDL strict-promotion runs)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_SEED_BASE` (integer `>=0`, default from policy file; aggregate VHDL strict-promotion deterministic seed base)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_SEED_STRIDE` (integer `>=1`, default from policy file; per-trial seed-base stride for VHDL strict-promotion runs)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default from policy file; parse-full mode forwarded to VHDL strict-promotion trials)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_REALISTIC_CORPUS_MODE` (`auto`/`0`/`1`, default from policy file; realistic-corpus mode forwarded to VHDL strict-promotion trials)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO` (`0-100`, default from policy file; parse-full ratio floor forwarded to VHDL strict-promotion trials)
- `PGEN_SOTA_VHDL_STRICT_PROMOTION_REQUIRE_REALISTIC_PARITY` (`0`/`1`, default from policy file; require realistic-corpus expected-vs-observed parity in VHDL strict-promotion trials)
- `PGEN_SOTA_ALLOW_INFORMATIONAL_FAILURES` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRED_CHECKS` (space-separated required check override list)
- `PGEN_SOTA_POLICY_FILE` (override machine policy file path)
- `PGEN_SOTA_EXIT_STATE_DIR` (override output state dir)

Aggregate promotion telemetry now includes replay-shadow primary-entry and alternate-entry counters for:
- `sv_declared_shadow_promotion_closed_loop_parseability_shadow_primary_entry_*`
- `sv_declared_shadow_promotion_closed_loop_parseability_shadow_alternate_entry_*`
- `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_primary_entry_*`
- `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_alternate_entry_*`
- `vhdl_strict_promotion_closed_loop_parseability_shadow_primary_entry_*`
- `vhdl_strict_promotion_closed_loop_parseability_shadow_alternate_entry_*`

Aggregate main-quality telemetry now includes both primary-entry and alternate-entry counters for:
- `sv_preprocessor_quality_target_drive_primary_entry_*`
- `sv_preprocessor_quality_target_drive_alternate_entry_*`
- `sv_stimuli_quality_closed_loop_parseability_shadow_primary_entry_*`
- `sv_stimuli_quality_closed_loop_parseability_shadow_alternate_entry_*`
- `vhdl_stimuli_quality_closed_loop_parseability_shadow_primary_entry_*`
- `vhdl_stimuli_quality_closed_loop_parseability_shadow_alternate_entry_*`

HDL readiness tuning:
- `PGEN_HDL_FRONTEND_STRICT` (`1`/`0`, default `0`)
- `PGEN_HDL_FRONTEND_STIMULI_COUNT` (default `8`)
- `PGEN_HDL_FRONTEND_STIMULI_SEED` (default `1337`)
- `PGEN_HDL_FRONTEND_STATE_DIR` (override output state dir, default `rust/target/hdl_frontend_gate`)

Release policy references:
- machine policy: `rust/config/sota_exit_policy.env`
- release checklist/policy doc: `PGEN_RELEASE_POLICY.md`
- stimuli-module normative contract: `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`

Optional robustness-gate tuning:
- `PGEN_ANNOTATION_ROBUSTNESS_COUNT` (default `32`)
- `PGEN_ANNOTATION_ROBUSTNESS_RETURN_SEED` (default `4242`)
- `PGEN_ANNOTATION_ROBUSTNESS_SEMANTIC_SEED` (default `4343`)

Optional non-annotation EBNF quality-gate tuning:
- `PGEN_EBNF_STIMULI_QUALITY_COUNT` (default `12`)
- `PGEN_EBNF_STIMULI_QUALITY_GAP_THRESHOLD` (default `1`)
- `PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` (default `5000`)
- `PGEN_EBNF_STIMULI_QUALITY_CONTRACT` (override grammar contract manifest path)

Optional SV preprocessor quality-gate tuning:
- `PGEN_SV_PREPROCESSOR_QUALITY_COUNT` (default `16`)
- `PGEN_SV_PREPROCESSOR_QUALITY_GAP_THRESHOLD` (default `1`)
- `PGEN_SV_PREPROCESSOR_QUALITY_TARGET_MAX_ATTEMPTS` (default `6000`)
- `PGEN_SV_PREPROCESSOR_QUALITY_SEED_BASE` (default `9101`)
- `PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS` (default `8`)
- `PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_SEED_START` (default `9201`)
- `PGEN_SV_PREPROCESSOR_QUALITY_VALIDATE_PARSEABILITY` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_PREPROCESSOR_DIFF_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_PREPROCESSOR_DIFF_MAX_SAMPLES` (default `4`)
- `PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER` (path to executable trusted-reference runner script; required for strict differential mode)
- `PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND` (`auto`/`iverilog`/`verilator`, consumed by the project runner shim)
- `PGEN_SV_PREPROCESSOR_REFERENCE_IVERILOG_BIN` (default `iverilog`)
- `PGEN_SV_PREPROCESSOR_REFERENCE_VERILATOR_BIN` (default `verilator`)
- `PGEN_SV_PREPROCESSOR_REFERENCE_LANGUAGE` (default `1800-2017`, used by `verilator` backend)
- `PGEN_SV_PREPROCESSOR_REFERENCE_INCLUDE_DIRS` (optional CSV include-dir list, consumed by runner shim)
- `PGEN_SV_PREPROCESSOR_REFERENCE_DEFINES` (optional CSV macro define list, consumed by runner shim)
- `PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR` (default `rust/target/sv_preprocessor_quality_gate`)
  - stage summary telemetry now includes:
    - `target_drive_primary_entry_attempts_total`
    - `target_drive_primary_entry_accepted_outputs_total`
    - `target_drive_primary_entry_rejected_outputs_total`
    - `target_drive_alternate_entry_attempts_total`
    - `target_drive_alternate_entry_accepted_outputs_total`
    - `target_drive_alternate_entry_rejected_outputs_total`

`sv_preprocessor_quality_gate` trusted-reference differential taxonomy:
- gate emits differential report JSON at:
  - `rust/target/sv_preprocessor_quality_gate/work/systemverilog_preprocessor_differential_report.json`
- when parseability is enabled, gate also emits aggregate parseability report JSON at:
  - `rust/target/sv_preprocessor_quality_gate/work/systemverilog_preprocessor_parseability_report.json`
- runner interface contract (`PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER`):
  - positional args:
    - `$1`: input SV sample file
    - `$2`: output preprocessed SV file
    - `$3`: output diagnostics JSON file
  - project-provided runner shim:
    - `rust/scripts/sv_preprocessor_reference_runner.sh`
    - backend auto-selection: `iverilog` first, then `verilator`
    - probe support:
      - `--probe` exits `0` when a backend is available, non-zero otherwise
      - `--help` advertises probe capability
    - always emits diagnostics as JSON array (empty `[]` on clean success)
- differential modes:
  - `0`: disabled
  - `auto`: enabled when runner is executable; otherwise report-only skip
  - `1`: strict (fails when runner unavailable or when any non-`match` taxonomy occurs)
- runner probe preflight behavior:
  - when runner supports `--probe`, gate executes probe before differential case classification
  - `auto`: probe failure downgrades mode to `unsupported_reference_runner` (taxonomy run skipped)
  - `1`: probe failure is immediate gate failure
- taxonomy categories:
  - `match`
  - `diagnostics_mismatch`
  - `whitespace_only_output_mismatch`
  - `output_mismatch`
  - `rust_failed_reference_passed`
  - `reference_failed_rust_passed`
  - `both_failed`
  - `reference_artifact_missing`
- strict differential example:
```bash
PGEN_SV_PREPROCESSOR_DIFF_MODE=1 \
PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER=$PWD/rust/scripts/sv_preprocessor_reference_runner.sh \
PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND=auto \
make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
```
- aggregate observability behavior (`sota_exit_gate`):
  - aggregate stage state dir:
    - `rust/target/sota_exit_gate/work/sv_preprocessor_quality_gate`
  - aggregate output and `summary.txt` include:
    - `sv_preprocessor_quality_state_dir`
    - `sv_preprocessor_quality_summary_csv`
    - `sv_preprocessor_quality_differential_report_json`
    - `sv_preprocessor_quality_parseability_mode_effective`
    - `sv_preprocessor_quality_diff_mode_effective`
    - `sv_preprocessor_quality_diff_mismatch_count`
    - `sv_preprocessor_quality_diff_taxonomy_output_mismatch`
    - `sv_preprocessor_quality_diff_taxonomy_rust_failed_reference_passed`
    - `sv_preprocessor_quality_diff_taxonomy_reference_failed_rust_passed`

`sv_preprocessor_curated_differential_gate` (offline expected-artifact differential):
- command:
```bash
make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate
```
- term definition:
  - `oracle model`: the source-of-truth used to decide pass/fail behavior.
  - `offline oracle model`: source-of-truth is local checked-in artifacts (no external tool dependency).
- oracle model:
  - compares Rust preprocessor output/diagnostics against checked-in expected artifacts from curated corpus.
  - does not require `iverilog` or `verilator`.
- tuning knobs:
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE` (`auto`/`0`/`1`, default `auto`)
    - `0`: disable curated differential stage
    - `auto`: run curated differential and report classification
    - `1`: strict mode (fails on `bug_mismatch`)
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_CORPUS` (default `rust/test_data/grammar_quality/systemverilog_preprocessor_curated_differential_corpus.json`)
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MAX_CASES` (default `0`, meaning all corpus cases)
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_STATE_DIR` (default `rust/target/sv_preprocessor_curated_differential_gate`)
- deterministic report artifact:
  - `rust/target/sv_preprocessor_curated_differential_gate/work/systemverilog_preprocessor_curated_differential_report.json`
- classification semantics:
  - `expected_match`: observed category is `match`
  - `expected_mismatch`: observed category is within case `expected_categories` but non-primary
  - `bug_mismatch`: observed category is outside case `expected_categories` and is treated as regression debt
- current curated corpus baseline:
  - `systemverilog_preprocessor_curated_differential_corpus.json` `version: 4`
  - 9 deterministic cases across positive and negative directive families:
    - define width
    - conditional branch
    - token paste
    - define/undef guard
    - nested conditionals
    - function-like macro arguments
    - local include expansion
    - include missing file (negative deterministic failure family)
    - include cycle (negative deterministic failure family)
  - category contract split:
    - positive stable families: `expected_categories: ["match"]`
    - deterministic negative families: `expected_categories: ["rust_failed_expected_passed"]`
  - expected strict-mode summary shape on current baseline:
    - `classification_expected_match=7`
    - `classification_expected_mismatch=2`
    - `classification_bug_mismatch=0`

`sv_preprocessor_template_differential_gate` (dynamic offline predictor):
- command:
```bash
make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate
```
- approach:
  - synthesizes deterministic SV preprocessor snippets from seed-driven templates,
  - predicts expected output/diagnostics from template logic (independent from runtime preprocessor path),
  - compares Rust preprocessor output vs predicted expected artifacts.
- default template families:
  - `template_define_width`
  - `template_ifdef_branch`
  - `template_token_paste`
  - `template_define_undef_ifdef`
  - `template_nested_ifdef`
  - `template_macro_function_args`
- tuning knobs:
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE` (`auto`/`0`/`1`, default `auto`)
    - `0`: disable template differential stage
    - `auto`: run template differential and report classification
    - `1`: strict mode (fails on `bug_mismatch`)
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_COUNT` (default `32`)
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_SEED_BASE` (default `13001`)
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_STATE_DIR` (default `rust/target/sv_preprocessor_template_differential_gate`)
- deterministic report artifact:
  - `rust/target/sv_preprocessor_template_differential_gate/work/systemverilog_preprocessor_template_differential_report.json`
- classification semantics:
  - `expected_match`: observed category is `match`
  - `expected_mismatch`: observed category is in expected set but non-primary (for example whitespace-only output drift)
  - `bug_mismatch`: observed category is outside expected set and is treated as regression debt
- diagnostics invariants:
  - expected diagnostics must be a JSON array.
  - observed diagnostics must be a JSON array (`diagnostics_contract_violation` taxonomy on contract failure).
  - warning/error counts are checked per case against expected values and aggregated as:
    - `diagnostics_invariant_pass_count`
    - `diagnostics_invariant_fail_count`

Optional SV stimuli quality-gate tuning:
- `PGEN_SV_STIMULI_QUALITY_CONTRACT` (default `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`)
- `PGEN_SV_STIMULI_QUALITY_COUNT` (override contract sample count)
- `PGEN_SV_STIMULI_QUALITY_SEED_BASE` (override contract seed base)
- `PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` (integer `>=1`, overrides contract `closed_loop.target_max_attempts` for the current invocation only)
  - intended for bounded evidence refreshes and faster closed-loop reruns without editing the contract manifest.
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO` (`0`/`1`, overrides contract parse-full ratio enforcement)
- `PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO` (`0-100`, overrides contract parse-full minimum pass ratio percent)
- `PGEN_SV_STIMULI_QUALITY_MODE` (`sv_file`/`sv_parseable_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default from contract)
- `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default `0`)
  - when set to `1` and `PGEN_SV_STIMULI_QUALITY_MODE` is unset, gate auto-selects `sv_semantic_file`.
- `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE` (`auto`/`0`/`1`, default `auto`)
  - `auto`: follow contract `semantic_promotion.declared_identifier_shadow_*` controls.
  - `0`: disable declared-identifier shadow burn-down telemetry.
  - `1`: strict trial mode (enable shadow checks and fail gate on any shadow failure).
- `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY` (`0`/`1`, default `0`)
  - `1`: run declared-shadow checks only on samples with `parse_full` status `pass`.
  - strict mode fails if parseable-only filtering leaves zero checked samples.
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE` (`auto`/`0`/`1`, default `auto`)
  - controls standalone `sv_declared_shadow_promotion_gate` behavior.
  - `auto`: run strict-trial matrix and emit recommendation report without failing on ineligible outcomes.
  - `0`: skip promotion-trial gate.
  - `1`: strict promotion mode (fails when runtime enforcement is not yet eligible).
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_TRIALS` (default `3`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT` (default `6`, sample count per trial)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE` (default `12001`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS` (default `400`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED` (default `2`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default `1`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE` (`sv_file`/`sv_parseable_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default `sv_file`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY` (`0`/`1`, default `1`)
- `PGEN_SV_DECLARED_SHADOW_PROMOTION_STATE_DIR` (default `rust/target/sv_declared_shadow_promotion_gate`)
  - standalone summary/report telemetry now includes:
    - `closed_loop_parseability_shadow_alternate_entry_attempts_total`
    - `closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_MODE` (`auto`/`0`/`1`, default `auto`)
  - controls standalone `sv_parse_full_ratio_promotion_gate` behavior.
  - `auto`: run strict-ratio trials and emit recommendation without failing on ineligible outcomes.
  - `0`: skip parse-full ratio promotion gate.
  - `1`: strict promotion mode (fails when threshold-ratchet eligibility is not met).
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS` (default `4`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_COUNT` (default `8`, sample count per trial)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE` (default `12001`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE` (default `250000`, per-trial seed-base offset)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default `0`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE` (`sv_file`/`sv_parseable_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default `sv_file`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO` (`0-100`, default `20`)
- `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STATE_DIR` (default `rust/target/sv_parse_full_ratio_promotion_gate`)
  - standalone summary/report telemetry now includes:
    - `closed_loop_parseability_shadow_alternate_entry_attempts_total`
    - `closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`
- `PGEN_SV_STIMULI_QUALITY_LRM_PROFILE` (single LRM profile override, for example `2017` or `2023`)
- `PGEN_SV_STIMULI_QUALITY_LRM_PROFILES` (CSV LRM profile matrix override, for example `2017,2023`)
- `PGEN_SV_STIMULI_QUALITY_DECLARED_IDENTIFIER_SUITE` (override declared-identifier deterministic contract corpus path)
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_DECLARED_IDENTIFIER_SUITE` (`0`/`1`, overrides contract enforcement toggle)
- `PGEN_SV_STIMULI_QUALITY_PORT_BINDING_SUITE` (override port-binding legality deterministic contract corpus path)
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_PORT_BINDING_SUITE` (`0`/`1`, overrides contract enforcement toggle)
- `PGEN_SV_STIMULI_QUALITY_PACKAGE_QUAL_SUITE` (override package-qualification deterministic contract corpus path)
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_PACKAGE_QUAL_SUITE` (`0`/`1`, overrides contract enforcement toggle)
- `PGEN_SV_STIMULI_QUALITY_CONTEXT_LEGALITY_SUITE` (override context-legality deterministic contract corpus path)
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_CONTEXT_LEGALITY_SUITE` (`0`/`1`, overrides contract enforcement toggle)
- `PGEN_SV_STIMULI_DIFF_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_STIMULI_DIFF_MAX_SAMPLES` (default `8`)
- `PGEN_SV_STIMULI_REFERENCE_RUNNER` (path to executable trusted-reference parser runner script; required for strict differential mode)
- `PGEN_SV_STIMULI_PERF_BUDGET_MODE` (`auto`/`0`/`1`, default `auto`)
  - `0`: disable SV performance/memory-proxy budget checks.
  - `auto`: follow contract toggle `performance_budgets.enforce`.
  - `1`: strict-enable budget checks regardless of contract toggle.
- `PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE` (`auto`/`0`/`1`, default `auto`)
  - `0`: disable Nexsim realistic-corpus integration/budget stage.
  - `auto`: follow contract toggle `nexsim_realistic_corpus.enforce`.
  - `1`: strict-enable realistic-corpus stage regardless of contract toggle.
- `PGEN_SV_STIMULI_REALISTIC_CORPUS` (override realistic-corpus manifest path)
- `PGEN_SV_STIMULI_REALISTIC_CORPUS_MAX_CASES` (integer `>=0`, default `0` meaning run all corpus cases)
- `PGEN_SV_STIMULI_QUALITY_STATE_DIR` (default `rust/target/sv_stimuli_quality_gate`)
  - standalone summary/report telemetry now includes:
    - `closed_loop_parseability_shadow_primary_entry_attempts_total`
    - `closed_loop_parseability_shadow_primary_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_primary_entry_rejected_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_attempts_total`
    - `closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`

Optional VHDL stimuli quality-gate tuning:
- `PGEN_VHDL_STIMULI_QUALITY_CONTRACT` (default `rust/test_data/grammar_quality/vhdl_core_v0_contract.json`)
- `PGEN_VHDL_STIMULI_QUALITY_COUNT` (override contract sample count)
- `PGEN_VHDL_STIMULI_QUALITY_SEED_BASE` (override contract seed base)
- `PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` (integer `>=1`, overrides contract `closed_loop.target_max_attempts` for the current invocation only)
- `PGEN_VHDL_STIMULI_QUALITY_PARSEABILITY_MAX_ATTEMPTS` (override contract `parseability_generation.max_attempts_per_sample`)
- `PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE` (`auto`/`0`/`1`, default `auto`)
  - `0`: disable VHDL realistic-corpus validation stage.
  - `auto`: follow contract toggle `realistic_corpus.enforce`.
  - `1`: strict-enable realistic-corpus validation regardless of contract toggle.
- `PGEN_VHDL_STIMULI_REALISTIC_CORPUS` (override realistic-corpus manifest path)
- `PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MAX_CASES` (integer `>=0`, default `0` meaning run all corpus cases)
- `PGEN_VHDL_STIMULI_QUALITY_STATE_DIR` (default `rust/target/vhdl_stimuli_quality_gate`)
  - standalone summary/report telemetry now includes:
    - `closed_loop_parseability_shadow_primary_entry_attempts_total`
    - `closed_loop_parseability_shadow_primary_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_primary_entry_rejected_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_attempts_total`
    - `closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`

Optional VHDL strict-promotion gate tuning:
- `PGEN_VHDL_STRICT_PROMOTION_MODE` (`auto`/`0`/`1`, default `auto`)
  - `auto`: run deterministic promotion trials and emit recommendation without failing on ineligible outcomes.
  - `0`: skip VHDL strict-promotion gate.
  - `1`: strict promotion mode (fails when required-strict eligibility is not met).
- `PGEN_VHDL_STRICT_PROMOTION_TRIALS` (default `3`)
- `PGEN_VHDL_STRICT_PROMOTION_COUNT` (default `8`, sample count per trial)
- `PGEN_VHDL_STRICT_PROMOTION_SEED_BASE` (default `22001`)
- `PGEN_VHDL_STRICT_PROMOTION_SEED_STRIDE` (default `250000`, per-trial seed-base offset)
- `PGEN_VHDL_STRICT_PROMOTION_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_VHDL_STRICT_PROMOTION_REALISTIC_CORPUS_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO` (`0-100`, default `0`)
- `PGEN_VHDL_STRICT_PROMOTION_REQUIRE_REALISTIC_PARITY` (`0`/`1`, default `1`)
- `PGEN_VHDL_STRICT_PROMOTION_STATE_DIR` (default `rust/target/vhdl_strict_promotion_gate`)
  - standalone summary/report telemetry now includes:
    - `closed_loop_parseability_shadow_alternate_entry_attempts_total`
    - `closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
    - `closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`

Optional SV syntax-closure gate tuning:
- `PGEN_SV_SYNTAX_CLOSURE_CONTRACT` (default `rust/test_data/grammar_quality/systemverilog_syntax_closure_contract.json`)
- `PGEN_SV_SYNTAX_CLOSURE_STATE_DIR` (default `rust/target/sv_syntax_closure_gate`)

`sv_stimuli_quality_gate` closed-loop stage contract:
- deterministic declared-identifier semantic contract precheck:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `semantic_contracts.declared_identifier_suite_path`
    - `semantic_contracts.enforce_declared_identifier_suite`
  - gate stage:
    - `declared_identifier_contract_suite`
  - summary metrics:
    - `declared_identifier_suite_status`
    - `declared_identifier_suite_total`
    - `declared_identifier_suite_passed`
    - `declared_identifier_suite_failed`
  - behavior:
    - runs before profile/sample generation loops,
    - enforces deterministic pass/fail expected outcomes from a fixed corpus (for example declared-vs-undeclared assignment/use, indexed-LHS assignment identifiers like `arr[idx] = ...`, `for`/`foreach` iterators, event-control and named-port usage).
    - declaration extraction includes `type` parameter declarations (for example `type T=...`) so semantic closure does not misclassify those identifiers as undeclared assignment uses.
- per-profile closed loop:
  - deterministic initial-stage replay check:
    - gate reruns initial closed-loop generation with same seed/profile config and asserts deterministic equivalence for:
      - stimuli text artifact
      - coverage JSON
      - gap JSON
      - gap text report
    - summary metric:
      - `closed_loop_initial_replay_determinism_passes`
  - effective replay-budget summary metrics:
    - `closed_loop_target_max_attempts`
    - `closed_loop_target_max_attempts_source`
      - `contract`: using `closed_loop.target_max_attempts` from the manifest.
      - `env_override`: using `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` for that invocation.
  - `coverage/gap(initial) -> target-driven replay -> non-increasing target debt check`.
  - preprocess convergence debt extraction on closed-loop corpora:
    - `closed_loop_initial_preprocess_warnings_total`
    - `closed_loop_initial_preprocess_errors_total`
    - `closed_loop_replay_preprocess_warnings_total`
    - `closed_loop_replay_preprocess_errors_total`
  - with `closed_loop.require_non_increasing_target_debt=true`, gate enforces:
    - `replay_targets <= initial_targets`
    - `replay_preprocess_errors <= initial_preprocess_errors` (per profile)
  - replay parseability shadow telemetry:
    - contract key:
      - `closed_loop.parseability_shadow_enabled`
    - behavior:
      - after the authoritative raw replay stage, the gate reruns the same target-driven replay seed/profile input in parser-backed generation mode,
      - parser-rejected replay-shadow outputs do not count as resolved rule/branch successes inside the shadow run,
      - the shadow stage is telemetry-only and does not alter:
        - `closed_loop_replay_targets_total`
        - non-increasing target debt enforcement
        - non-increasing preprocess error debt enforcement
    - summary metrics:
      - `closed_loop_parseability_shadow_enabled`
      - `closed_loop_parseability_shadow_effective`
      - `closed_loop_parseability_shadow_note`
      - `closed_loop_parseability_shadow_requested_total`
      - `closed_loop_parseability_shadow_attempts_total`
      - `closed_loop_parseability_shadow_accepted_total`
      - `closed_loop_parseability_shadow_rejected_total`
      - `closed_loop_parseability_shadow_parser_rejections_total`
      - `closed_loop_parseability_shadow_generation_errors_total`
      - `closed_loop_parseability_shadow_empty_generations_total`
      - `closed_loop_parseability_shadow_acceptance_rate_percent`
      - `closed_loop_parseability_shadow_report_json`
    - focused evidence:
      - `PGEN_SV_STIMULI_QUALITY_COUNT=1`
      - `PGEN_SV_STIMULI_DIFF_MODE=0`
      - `PGEN_SV_STIMULI_PERF_BUDGET_MODE=0`
      - `PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0`
      - `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400`
      - observed:
        - authoritative replay debt: `4876 -> 3925`
        - replay parseability shadow: `requested_total=491`, `accepted_total=148`, `rejected_total=343`, `acceptance_rate_percent=30.14`
- per-sample deterministic flow:
  - `stimuli_generate -> preprocess -> parse_full(optional) -> semantic_validate_baseline`.
- trusted-reference differential taxonomy:
  - gate emits parser differential report JSON at:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_differential_report.json`
  - runner interface contract (`PGEN_SV_STIMULI_REFERENCE_RUNNER`):
    - positional args:
      - `$1`: preprocessed SV sample file
      - `$2`: output AST JSON file path (runner may emit placeholder JSON)
      - `$3`: output diagnostics JSON file path (must be JSON array; `[]` allowed)
    - exit code contract:
      - `0`: reference parser accepted input
      - non-zero: reference parser rejected input / failed parse
  - differential modes:
    - `0`: disabled
    - `auto`: enabled when runner is executable and mode is parse-full eligible
    - `1`: strict (fails when prerequisites are missing or when asymmetric parseability mismatches occur)
  - taxonomy categories:
    - `match`
    - `rust_failed_reference_passed`
    - `reference_failed_rust_passed`
    - `both_failed`
    - `reference_artifact_missing`
  - strict differential example:
```bash
PGEN_SV_STIMULI_DIFF_MODE=1 \
PGEN_SV_STIMULI_REFERENCE_RUNNER=/abs/path/to/reference_parser_runner.sh \
make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
```
- deterministic performance/memory-proxy budget stage:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `performance_budgets.enforce`
    - `performance_budgets.max_generate_ms_per_sample`
    - `performance_budgets.max_preprocess_ms_per_sample`
    - `performance_budgets.max_parse_full_ms_per_sample`
    - `performance_budgets.max_sample_bytes`
    - `performance_budgets.max_preprocessed_bytes`
  - mode control:
    - `PGEN_SV_STIMULI_PERF_BUDGET_MODE=auto|0|1`
  - deterministic report artifact:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_performance_report.json`
  - summary metrics include:
    - effective mode/note,
    - threshold values,
    - observed per-stage totals/averages/maxima,
    - max generated/preprocessed sample size.
- deterministic realistic-corpus budget/integration stage:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `nexsim_realistic_corpus.enforce`
    - `nexsim_realistic_corpus.cases_path`
    - `nexsim_realistic_corpus.max_preprocess_ms_per_case`
    - `nexsim_realistic_corpus.max_parse_full_ms_per_case`
    - `nexsim_realistic_corpus.max_sample_bytes`
    - `nexsim_realistic_corpus.max_preprocessed_bytes`
    - `nexsim_realistic_corpus.require_no_preprocess_errors`
  - mode/path controls:
    - `PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=auto|0|1`
    - `PGEN_SV_STIMULI_REALISTIC_CORPUS`
    - `PGEN_SV_STIMULI_REALISTIC_CORPUS_MAX_CASES`
  - deterministic report artifact:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_nexsim_realistic_corpus_report.json`
  - behavior:
    - runs curated SV fixtures through `preprocess -> parse_full` with per-case timing/size budget enforcement,
    - `expect_parse_full_pass=true` means parse-full pass is required,
    - `expect_parse_full_pass=false` means parse-full fail is currently accepted (a pass is tracked as improvement telemetry, not a failure),
    - report captures case-level outcomes plus aggregate timing/size/diagnostic totals.
  - current checked-in baseline (`version: 3`):
    - `11` declared cases (`22` executions across `2017` + `2023`),
    - `11` expected-pass families,
    - `0` expected-fail realism sentinels.
  - supported expected-pass families currently include:
    - module ports + assignment,
    - named instantiation (single-port and multi-port),
    - package-qualified types and package-qualified constant/vector-width references,
    - interface/modport,
    - wildcard instantiation (`.*`),
    - file-level `timeunit`,
    - `always_ff` sequential blocks with edge-qualified event control,
    - generate-`for` indexed continuous assignment (`assign y[i] = a[i];`).
- aggregate observability behavior (`sota_exit_gate`):
  - aggregate stage state dir:
    - `rust/target/sota_exit_gate/work/sv_stimuli_quality_gate`
  - aggregate output and `summary.txt` include:
    - `sv_stimuli_quality_state_dir`
    - `sv_stimuli_quality_parse_full_quality_report_json`
    - `sv_stimuli_quality_parse_full_pass_ratio_percent`
    - `sv_stimuli_quality_diff_report_json`
    - `sv_stimuli_quality_diff_mismatch_count`
    - `sv_stimuli_quality_performance_report_json`
    - `sv_stimuli_quality_performance_enabled`
    - `sv_stimuli_quality_closed_loop_initial_targets_total`
    - `sv_stimuli_quality_closed_loop_replay_targets_total`
    - `sv_stimuli_quality_closed_loop_parseability_shadow_*`
    - `sv_stimuli_quality_parseability_generation_*`
- deterministic port-binding legality semantic contract precheck:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `semantic_contracts.port_binding_legality_suite_path`
    - `semantic_contracts.enforce_port_binding_legality_suite`
  - gate stage:
    - `port_binding_legality_contract_suite`
  - summary metrics:
    - `port_binding_suite_status`
    - `port_binding_suite_total`
    - `port_binding_suite_passed`
    - `port_binding_suite_failed`
  - behavior:
    - runs before profile/sample generation loops,
    - enforces deterministic pass/fail outcomes for named-port legality on known module declarations (`module_type.port`) independent of random stimuli variance.
- deterministic package-qualification semantic contract precheck:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `semantic_contracts.package_qualification_suite_path`
    - `semantic_contracts.enforce_package_qualification_suite`
  - gate stage:
    - `package_qualification_contract_suite`
  - summary metrics:
    - `package_qualification_suite_status`
    - `package_qualification_suite_total`
    - `package_qualification_suite_passed`
    - `package_qualification_suite_failed`
  - behavior:
    - runs before profile/sample generation loops,
    - enforces deterministic pass/fail outcomes for package qualification resolution checks (`pkg::symbol`) independent of random stimuli variance.
- deterministic context-legality semantic contract precheck:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `semantic_contracts.context_legality_suite_path`
    - `semantic_contracts.enforce_context_legality_suite`
  - gate stage:
    - `context_legality_contract_suite`
  - summary metrics:
    - `context_legality_suite_status`
    - `context_legality_suite_total`
    - `context_legality_suite_passed`
    - `context_legality_suite_failed`
  - behavior:
    - runs before profile/sample generation loops,
    - enforces deterministic pass/fail outcomes for baseline context legality (`always_comb` event-control prohibition, `always_ff` nonblocking requirement, generate `for` iterator `genvar` declaration).
- declared-identifier promotion burn-down shadow telemetry:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `semantic_promotion.declared_identifier_shadow_enabled`
    - `semantic_promotion.declared_identifier_shadow_strict`
  - env override:
    - `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE=auto|0|1`
  - deterministic report artifact:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_declared_identifier_shadow_report.json`
  - summary metrics:
    - `declared_shadow_effective`
    - `declared_shadow_checked`
    - `declared_shadow_passed`
    - `declared_shadow_failed`
  - behavior:
    - when effective semantic policy leaves `require_declared_identifiers_before_use=false` (for example `sv_file` profile), gate runs per-sample shadow checks for promotion evidence,
    - strict mode (`1` or strict contract key) turns shadow failures into gate failures for controlled promotion trials.
- declared-shadow promotion trial gate:
  - target:
    - `make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate`
  - deterministic report artifact:
    - `rust/target/sv_declared_shadow_promotion_gate/work/systemverilog_declared_identifier_promotion_report.json`
  - report fields:
    - `recommendation` (`enable_runtime_declared_identifiers` or `hold`)
    - `eligibility.eligible_for_runtime_enforcement`
    - `declared_shadow_parseable_only` (effective parseability scope forwarded to strict declared-shadow trial checks)
    - aggregated strict-trial totals (`checked/passed/failed`)
    - aggregated parser-backed effort telemetry:
      - `parseability_generation.observed.attempts_total`
      - `parseability_generation.observed.accepted_total`
      - `parseability_generation.observed.rejected_total`
      - `parseability_generation.observed.acceptance_rate_percent`
      - `closed_loop_parseability_shadow.observed.attempts_total`
      - `closed_loop_parseability_shadow.observed.accepted_total`
      - `closed_loop_parseability_shadow.observed.rejected_total`
      - `closed_loop_parseability_shadow.observed.acceptance_rate_percent`
    - `totals.skipped_unparseable` from parseable-only filtering in underlying strict trials
    - per-trial blocker attribution:
      - `trials[].blocker_key`
      - `trials[].blocker_detail`
    - per-trial parser-backed effort telemetry:
      - `trials[].parseability_generation`
      - `trials[].closed_loop_parseability_shadow`
    - aggregate blocker attribution:
      - `blockers.failed_trial_count`
      - `blockers.non_shadow_blocked_trial_count`
      - `blockers.primary_non_shadow_blocker`
      - `blockers.breakdown`
      - `blockers.non_shadow_breakdown`
    - per-trial logs and shadow-report references
  - default trial profile:
    - uses `sv_file` mode with parseability-scoped shadow checks to isolate declared-before-use promotion evidence from unrelated semantic-closure blockers.
  - default aggregate policy:
    - wired into `sota_exit_gate` as required strict (`run=1`, `strict=1`) after promotion-trial baseline convergence.
    - trial shape is policy-driven via:
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TRIALS`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_COUNT`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE`
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY`
      plus matching `PGEN_SOTA_SV_*` runtime overrides.
  - aggregate observability behavior:
    - when run from `sota_exit_gate`, declared-shadow artifacts are written under:
      - `rust/target/sota_exit_gate/work/sv_declared_shadow_promotion_gate`
    - aggregate output and `summary.txt` include:
      - `sv_declared_shadow_promotion_report_json`
      - `sv_declared_shadow_promotion_recommendation`
      - `sv_declared_shadow_promotion_eligible_for_runtime_enforcement`
      - `sv_declared_shadow_promotion_totals_failed`
      - `sv_declared_shadow_promotion_totals_checked`
      - `sv_declared_shadow_promotion_primary_non_shadow_blocker`
      - `sv_declared_shadow_promotion_declared_shadow_parseable_only`
      - `sv_declared_shadow_promotion_failed_trial_count`
      - `sv_declared_shadow_promotion_non_shadow_blocked_trial_count`
      - `sv_declared_shadow_promotion_parseability_generation_attempts_total`
      - `sv_declared_shadow_promotion_parseability_generation_accepted_total`
      - `sv_declared_shadow_promotion_parseability_generation_rejected_total`
      - `sv_declared_shadow_promotion_parseability_generation_acceptance_rate_percent`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_attempts_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_accepted_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_rejected_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_acceptance_rate_percent`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_primary_entry_attempts_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_primary_entry_accepted_outputs_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_primary_entry_rejected_outputs_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_alternate_entry_attempts_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
      - `sv_declared_shadow_promotion_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`.
- parse-full ratio promotion trial gate:
  - target:
    - `make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
  - deterministic report artifact:
    - `rust/target/sv_parse_full_ratio_promotion_gate/work/systemverilog_parse_full_ratio_promotion_report.json`
  - report fields:
    - `recommendation` (`raise_min_parse_full_pass_ratio` or `hold`)
    - `eligibility.eligible_for_ratio_promotion`
    - per-trial parse-full ratio outcomes and aggregated min/max/avg ratio telemetry
    - aggregated parser-backed effort telemetry:
      - `parseability_generation.observed.attempts_total`
      - `parseability_generation.observed.accepted_total`
      - `parseability_generation.observed.rejected_total`
      - `parseability_generation.observed.acceptance_rate_percent`
      - `closed_loop_parseability_shadow.observed.attempts_total`
      - `closed_loop_parseability_shadow.observed.accepted_total`
      - `closed_loop_parseability_shadow.observed.rejected_total`
      - `closed_loop_parseability_shadow.observed.acceptance_rate_percent`
    - per-trial blocker attribution:
      - `trials[].blocker_key`
      - `trials[].blocker_detail`
    - per-trial parser-backed effort telemetry:
      - `trials[].parseability_generation`
      - `trials[].closed_loop_parseability_shadow`
    - aggregate blocker attribution:
      - `blockers.failed_trial_count`
      - `blockers.non_ratio_blocked_trial_count`
      - `blockers.primary_non_ratio_blocker`
      - `blockers.breakdown`
      - `blockers.non_ratio_breakdown`
  - behavior:
    - runs strict `sv_stimuli_quality_gate` trials at target ratio threshold to determine if aggregate minimum can be ratcheted safely.
  - default trial profile:
    - aligned to aggregate enforcement surface (`sv_file`, semantic closure disabled) so recommendation debt reflects parse-full ratio only.
  - blocker interpretation:
    - ratio debt appears as `parse_full_ratio_threshold_not_met`,
    - non-ratio blockers (for example `semantic_baseline_validation_failed`) are surfaced explicitly for targeted remediation.
  - default aggregate policy:
    - wired into `sota_exit_gate` as informational-first (`run=1`, `strict=0`) while ratchet evidence converges.
    - target threshold is policy-driven via:
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO`
      - runtime override `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO`.
    - trial shape is also policy-driven via:
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE`
      plus matching `PGEN_SOTA_SV_*` runtime overrides.
  - aggregate observability behavior:
    - when run from `sota_exit_gate`, promotion artifacts are written under:
      - `rust/target/sota_exit_gate/work/sv_parse_full_ratio_promotion_gate`
    - aggregate output prints:
      - `sv_parse_full_ratio_promotion_report_json`
      - `sv_parse_full_ratio_promotion_recommendation`
      - `sv_parse_full_ratio_promotion_primary_non_ratio_blocker`
      - `sv_parse_full_ratio_promotion_observed_ratio_min`
      - `sv_parse_full_ratio_promotion_observed_ratio_max`
      - `sv_parse_full_ratio_promotion_observed_ratio_avg`
      - `sv_parse_full_ratio_promotion_failed_trial_count`
      - `sv_parse_full_ratio_promotion_non_ratio_blocked_trial_count`
      - `sv_parse_full_ratio_promotion_parseability_generation_attempts_total`
      - `sv_parse_full_ratio_promotion_parseability_generation_accepted_total`
      - `sv_parse_full_ratio_promotion_parseability_generation_rejected_total`
      - `sv_parse_full_ratio_promotion_parseability_generation_acceptance_rate_percent`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_attempts_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_accepted_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_rejected_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_acceptance_rate_percent`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_primary_entry_attempts_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_primary_entry_accepted_outputs_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_primary_entry_rejected_outputs_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_alternate_entry_attempts_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
      - `sv_parse_full_ratio_promotion_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`.
    - aggregate summary artifact also persists these fields in:
      - `rust/target/sota_exit_gate/summary.txt`
      under section:
      - `Promotion Telemetry`.
- VHDL strict-promotion trial gate:
  - target:
    - `make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate`
  - deterministic report artifact:
    - `rust/target/vhdl_strict_promotion_gate/work/vhdl_strict_promotion_report.json`
  - report fields:
    - `recommendation` (`enable_required_strict_mode` or `hold`)
    - `eligibility.eligible_for_required_strict_mode`
    - per-trial parse-full ratio outcomes and aggregated min/max/avg ratio telemetry
    - aggregated parser-backed effort telemetry:
      - `parseability_generation.observed.attempts_total`
      - `parseability_generation.observed.accepted_total`
      - `parseability_generation.observed.rejected_total`
      - `parseability_generation.observed.acceptance_rate_percent`
      - `closed_loop_parseability_shadow.observed.attempts_total`
      - `closed_loop_parseability_shadow.observed.accepted_total`
      - `closed_loop_parseability_shadow.observed.rejected_total`
      - `closed_loop_parseability_shadow.observed.acceptance_rate_percent`
    - per-trial blocker attribution:
      - `trials[].blocker_key`
      - `trials[].blocker_detail`
    - per-trial parser-backed effort telemetry:
      - `trials[].parseability_generation`
      - `trials[].closed_loop_parseability_shadow`
    - aggregate blocker attribution:
      - `blockers.failed_trial_count`
      - `blockers.primary_blocker`
      - `blockers.breakdown`
  - behavior:
    - runs deterministic `vhdl_stimuli_quality_gate` trial matrix and emits readiness recommendation for promoting aggregate VHDL mode to required strict.
    - supports strict mode (`PGEN_VHDL_STRICT_PROMOTION_MODE=1`) and is now policy-enforced in aggregate required mode.
  - default aggregate policy:
    - wired into `sota_exit_gate` as required strict (`run=1`, `strict=1`).
    - trial shape and constraints are policy-driven via:
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_TRIALS`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_COUNT`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_SEED_BASE`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_SEED_STRIDE`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_PARSE_FULL_MODE`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_REALISTIC_CORPUS_MODE`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO`
      - `PGEN_SOTA_POLICY_VHDL_STRICT_PROMOTION_REQUIRE_REALISTIC_PARITY`
      plus matching `PGEN_SOTA_VHDL_*` runtime overrides.
  - aggregate observability behavior:
    - when run from `sota_exit_gate`, promotion artifacts are written under:
      - `rust/target/sota_exit_gate/work/vhdl_strict_promotion_gate`
    - aggregate output and `summary.txt` include:
      - `vhdl_strict_promotion_report_json`
      - `vhdl_strict_promotion_recommendation`
      - `vhdl_strict_promotion_eligible_for_required_strict_mode`
      - `vhdl_strict_promotion_primary_blocker`
      - `vhdl_strict_promotion_trial_passed`
      - `vhdl_strict_promotion_trial_failed`
      - `vhdl_strict_promotion_parseability_generation_attempts_total`
      - `vhdl_strict_promotion_parseability_generation_accepted_total`
      - `vhdl_strict_promotion_parseability_generation_rejected_total`
      - `vhdl_strict_promotion_parseability_generation_acceptance_rate_percent`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_attempts_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_accepted_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_rejected_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_acceptance_rate_percent`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_primary_entry_attempts_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_primary_entry_accepted_outputs_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_primary_entry_rejected_outputs_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_alternate_entry_attempts_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total`
      - `vhdl_strict_promotion_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total`.
- profile behavior:
  - contract defines supported/required LRM profiles (`2017`, `2023`) for one common `systemverilog.ebnf`,
  - runtime profile selection now activates real grammar-profile filtering (`sv_2017`, `sv_2023`) rather than metadata-only aliases,
  - gate executes selected profile set and reports profile-tagged rows in summary output.
- stimuli mode behavior (from `systemverilog_core_v0_contract.json` `stimuli_modes`):
  - `sv_file`:
    - entry rule: `systemverilog_file`,
    - closed-loop enabled by default,
    - parse-full eligible,
    - default recovery stimuli mode: `baseline`.
  - `sv_parseable_file`:
    - entry rule: `systemverilog_parseable_file`,
    - parse-full burn-down focused subset mode,
    - closed-loop enabled by default,
    - parse-full eligible,
    - default recovery stimuli mode: `baseline`.
  - `sv_snippet`:
    - entry rule: `source_item`,
    - closed-loop disabled by default,
    - parse-full ineligible (auto mode skips parse-full; strict parse-full mode errors out),
    - default recovery stimuli mode: `near_sync_negative`.
  - `sv_pp_file`:
    - entry rule: `systemverilog_file`,
    - preprocess-aware file mode,
    - closed-loop enabled by default,
    - parse-full eligible,
    - default recovery stimuli mode: `recovery_biased`.
  - `sv_pp_snippet`:
    - entry rule: `source_item`,
    - preprocess-aware snippet mode,
    - closed-loop disabled by default,
    - parse-full ineligible (auto mode skips parse-full; strict parse-full mode errors out),
    - default recovery stimuli mode: `near_sync_negative`.
  - `sv_semantic_file`:
    - entry rule: `systemverilog_file`,
    - semantic-closure focused file mode,
    - closed-loop enabled by default,
    - parse-full eligible,
    - default recovery stimuli mode: `baseline`,
    - mode semantic overrides currently enable:
      - `require_port_binding_legality_basic=true`
      - `require_declared_identifiers_before_use=true`
      - `require_declared_identifiers_parseable_only=true`
      - `require_package_qualification_resolution=true`
      - `require_width_compatibility_simple=true`
      - `require_context_legality_basic=true`
  - mode-level recovery steering:
    - optional profile key:
      - `stimuli_modes.profiles.<mode>.recovery_stimuli_mode`
    - allowed values:
      - `baseline`
      - `recovery_biased`
      - `near_sync_negative`
    - gate forwards this to all mode-run stimuli generation calls via `--recovery-stimuli-mode`.
  - word-boundary spacing control:
    - `ast_pipeline` supports `--enforce-word-boundary-spacing` for stimuli generation modes.
    - when enabled, spacing policy applies to:
      - terminal `\\b` regex candidates,
      - sequence/quantified fragment concatenation where adjacent generated segments would fuse lexical words.
    - `sv_stimuli_quality_gate` enables this control on all generation stages to reduce fused keyword/identifier artifacts before preprocess/parse_full checks.
  - mode-level generation bounds:
    - optional profile keys:
      - `stimuli_modes.profiles.<mode>.max_depth`
      - `stimuli_modes.profiles.<mode>.max_repeat`
    - gate behavior:
      - validates both values as integers `>= 1`,
      - forwards them to all generation calls as:
        - `--max-depth`
        - `--max-repeat`.
    - current contract baseline:
      - `sv_file` uses conservative caps:
        - `max_depth=20`
        - `max_repeat=2`
      to keep full-file stimuli parseable while preserving deterministic coverage/gap loop behavior.
  - mode-level semantic overrides:
    - optional profile key:
      - `stimuli_modes.profiles.<mode>.semantic_overrides.<semantic_baseline_toggle>`
    - overrides are applied after global `semantic_baseline` defaults to compute effective per-mode semantic checks.
    - current contract policy:
      - `sv_file`: `require_port_binding_legality_basic=true`
      - `sv_parseable_file`:
        - `require_port_binding_legality_basic=false`
        - `require_declared_identifiers_before_use=false`
        - `require_package_qualification_resolution=false`
        - `require_width_compatibility_simple=false`
        - `require_context_legality_basic=false`
      - `sv_snippet`: `require_port_binding_legality_basic=false`
      - `sv_pp_file`: `require_port_binding_legality_basic=true`
      - `sv_pp_snippet`: `require_port_binding_legality_basic=false`
      - `sv_semantic_file`:
        - `require_port_binding_legality_basic=true`
        - `require_declared_identifiers_before_use=true`
        - `require_declared_identifiers_parseable_only=true`
        - `require_package_qualification_resolution=true`
        - `require_width_compatibility_simple=true`
        - `require_context_legality_basic=true`
- closed-loop contract controls (from `systemverilog_core_v0_contract.json`):
  - top-level `sample_count` (current default `8`)
  - `closed_loop.gap_report_threshold`
  - `closed_loop.target_max_attempts`
    - runtime override:
      - `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS`
  - `closed_loop.replay_sample_count` (current default `8`)
  - `closed_loop.require_non_increasing_target_debt`
- failure replay + shrinking controls (from `systemverilog_core_v0_contract.json`):
  - `failure_replay.enabled`
  - `failure_replay.shrink_semantic_failures`
  - `failure_replay.shrink_parse_full_failures`
  - `failure_replay.shrink_max_iterations`
  - behavior:
    - on semantic failure, gate can emit deterministic shrunk artifact (`*.semantic.shrunk.sv`) preserving failing semantic predicate.
    - on parse-full rejection, gate can emit deterministic shrunk artifact (`*.parse_full.shrunk.sv`) preserving parser rejection.
    - summary reports shrink counters:
      - `semantic_failures_shrunk`
      - `parse_full_failures_shrunk`
- semantic baseline is currently:
  - non-empty preprocessed output,
  - no `error` severity in preprocessor diagnostics.
  - no duplicate named-port bindings in the same statement (`semantic_baseline.require_unique_named_port_bindings`).
  - optional basic named-port legality check against known in-file module headers (`semantic_baseline.require_port_binding_legality_basic`).
  - optional structural keyword-balance check (`semantic_baseline.require_balanced_structural_keywords`, currently disabled in default contract due high false-positive risk on current random samples).
  - optional declaration-before-use heuristic (`semantic_baseline.require_declared_identifiers_before_use`).
    - current implementation uses structured use-site scanning (assignments, conditions, event controls, named-port actuals), strips quoted strings/directives, ignores member/namespace/macro contexts, and handles additional declaration contexts (ports/imports/for/foreach/instantiation).
    - semantic-closure profile (`sv_semantic_file`) now enables this check with parseability guardrails:
      - `require_declared_identifiers_parseable_only=true` skips this runtime check when `parse_full` status is not `pass`.
    - deterministic contract coverage exists in `rust/test_data/grammar_quality/systemverilog_declared_identifier_contract_cases.json`, including explicit `foreach (arr[idx])` iterator declaration handling and preprocess-heavy conditional-directive families.
  - optional package qualification/import resolution heuristic (`semantic_baseline.require_package_qualification_resolution`).
    - deterministic contract coverage in `rust/test_data/grammar_quality/systemverilog_package_qualification_contract_cases.json` includes macro-qualified package reference pass/fail families.
  - optional simple packed-width vs literal-width compatibility check (`semantic_baseline.require_width_compatibility_simple`).
    - current implementation covers packed declarations of `logic|reg|wire|bit` and indexed LHS assignment forms.
    - deterministic contract coverage in `rust/test_data/grammar_quality/systemverilog_width_compatibility_contract_cases.json` includes preprocess-conditional overflow families.
  - optional basic context legality checks (`semantic_baseline.require_context_legality_basic`):
    - `always_comb` must not contain event controls,
    - `always_ff` must not contain blocking assignments,
    - generate `for` iterators must be declared `genvar`.
    - deterministic contract coverage in `rust/test_data/grammar_quality/systemverilog_context_legality_contract_cases.json` includes directive-noise and preprocess-conditional `always_ff` families.
  - optional basic named-port legality checks (`semantic_baseline.require_port_binding_legality_basic`) are backed by deterministic contract coverage in `rust/test_data/grammar_quality/systemverilog_port_binding_legality_contract_cases.json`, including preprocess-conditional unknown-binding failure families.
  - initial annotation-driven SV stimuli steering is now embedded directly in `grammars/systemverilog.ebnf`:
    - top-level/statement branch steering via `@branch_policy: priority_first` + `@priority: [...]`,
    - top-level coverage hints via `@coverage_target` + `@critical_path`,
    - token-family steering via `@token_class` on `simple_identifier`/`integral_number`.
    This is an initial baseline; broader rule-level semantic steering rollout remains in progress.
  - rule-level expansion now also covers additional high-fanout rules:
    - item/declaration flow: `module_item`, `non_port_module_item`, `program_item`, `non_port_program_item`, `module_or_generate_item`, `interface_or_generate_item`, `package_or_generate_item_declaration`, `generate_item`, `block_item_declaration`, `statement_or_null`,
    - lexical/shape flow: `module_keyword`, `module_header_ports`, `named_port_connection`, `hierarchy_separator`, `primary`, `data_type`, `identifier`.
  - module-header/parameter steering closure details:
    - `module_header_ports` now prioritizes `list_of_port_declarations` over `dot_star`/empty alternatives,
    - `parameter_port_list` now uses explicit `priority_first` steering to bias populated declaration forms over empty `#()`,
    - `parameter_port_declaration` now has explicit branch priorities favoring stable declaration branches.
  - mode-closure validation coverage:
    - representative deterministic runs are green for:
      - `sv_file` (parse-full enabled),
      - `sv_parseable_file` (parse-full burn-down subset),
      - `sv_semantic_file` (semantic-closure profile),
      - `sv_snippet` (expected parse-full ineligible path).
  - important steering detail:
    - `identifier` steering now explicitly favors `simple_identifier` over `escaped_identifier` to reduce exotic-name noise in generated SV stimuli.
- parse-full stage behavior:
  - `auto`: gate builds a temporary `systemverilog` adapter from the generated parser artifact and runs parse-full when available; parse-full rejections are recorded as soft-fail stage entries (gate continues),
  - `0`: disabled,
  - `1`: required and strict (fails gate if adapter is unavailable or if any sample parse-full rejects).
- parse-full acceptance quality contract:
  - contract keys (`systemverilog_core_v0_contract.json`):
    - `parse_full_quality.enforce_min_pass_ratio`
    - `parse_full_quality.min_pass_ratio`
  - deterministic report artifact:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_parse_full_quality_report.json`
  - summary metrics:
    - `parse_full_quality_enforced`
    - `parse_full_quality_effective`
    - `parse_full_quality_min_pass_ratio`
    - `parse_full_pass_ratio_percent`
  - behavior:
    - when enforcement is disabled, parse-full pass ratio is reported as telemetry,
    - when enforcement is enabled and parse-full is unavailable, gate fails,
    - when enforcement is enabled and ratio is below threshold, gate fails.
  - aggregate policy default:
    - `sota_exit_gate` forwards parse-full quality controls into `sv_stimuli_quality_gate` with policy defaults:
      - `PGEN_SOTA_POLICY_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1`
      - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=100`
    - next promotion-trial target (informational ratchet evidence):
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100`
    - current promotion-trial evidence density:
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS=4`
      - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT=8`

`vhdl_stimuli_quality_gate` closed-loop stage contract:
- deterministic flow:
  - `EBNF -> JSON -> parser -> coverage/gap(initial) -> target-driven replay -> parser-backed sample generation telemetry -> parse_full(optional)`.
- contract controls (from `vhdl_core_v0_contract.json`):
  - `entry_rule`
  - `sample_count`
  - `seed_base`
  - `closed_loop.gap_report_threshold`
  - `closed_loop.target_max_attempts`
    - runtime override:
      - `PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS`
  - `closed_loop.replay_sample_count`
  - `closed_loop.require_non_increasing_target_debt`
  - `closed_loop.parseability_shadow_enabled`
  - `parseability_generation.enabled`
  - `parseability_generation.max_attempts_per_sample`
- summary text now also emits:
  - `closed_loop_target_max_attempts`
  - `closed_loop_target_max_attempts_source`
    - `contract`: using `closed_loop.target_max_attempts` from the manifest.
    - `env_override`: using `PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` for that invocation.
- aggregate `sota_exit_gate` also exposes:
  - `PGEN_SOTA_POLICY_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS`
  - `PGEN_SOTA_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS`
  - summary fields:
    - `vhdl_stimuli_quality_target_max_attempts`
    - `vhdl_stimuli_quality_target_max_attempts_source`
      - `policy`: using the aggregate policy/runtime surface.
      - `runtime_override`: using `PGEN_SOTA_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS` for that aggregate invocation.
    - `vhdl_stimuli_quality_closed_loop_target_max_attempts`
    - `vhdl_stimuli_quality_closed_loop_target_max_attempts_source`
  - the same aggregate VHDL replay budget is inherited by `vhdl_strict_promotion_gate`.
- closed-loop replay parseability shadow:
  - when `closed_loop.parseability_shadow_enabled=true`, the gate runs an additional parser-backed replay command using the same target report and seed as the authoritative raw replay stage.
  - this shadow run is telemetry-only:
    - it does not replace the authoritative debt check,
    - it does not change `closed_loop_replay_targets`,
    - it exists to measure how much of the target-driven replay output remains parseable under the generated parser.
  - inside the shadow run, parser-rejected outputs do not count as resolved successes for the target plan; the generator retains branch-selection history but rolls back rule/branch success debt on rejected candidates.
  - that retained history now also drives a generic low-yield branch throttle, so replay-shadow and future parser-backed target loops can back off branches that consume many attempts per accepted output.
  - summary text now emits:
    - `closed_loop_parseability_shadow_requested_total`
    - `closed_loop_parseability_shadow_attempts_total`
    - `closed_loop_parseability_shadow_accepted_total`
    - `closed_loop_parseability_shadow_rejected_total`
    - `closed_loop_parseability_shadow_parser_rejections_total`
    - `closed_loop_parseability_shadow_generation_errors_total`
    - `closed_loop_parseability_shadow_empty_generations_total`
    - `closed_loop_parseability_shadow_acceptance_rate_percent`
    - `closed_loop_parseability_shadow_report_json`
- parser-backed sample-generation telemetry:
  - when parse-full is enabled and `parseability_generation.enabled=true`, the sampled stage uses the shared `ast_pipeline --validate-parseability --parseability-report-json` path before the explicit parse-full probe.
  - aggregate summary now emits:
    - `parseability_generation_requested_total`
    - `parseability_generation_attempts_total`
    - `parseability_generation_accepted_total`
    - `parseability_generation_rejected_total`
    - `parseability_generation_parser_rejections_total`
    - `parseability_generation_errors_total`
    - `parseability_generation_empty_generations_total`
    - `parseability_generation_acceptance_rate_percent`
    - `parseability_generation_report_json`
  - deterministic aggregate artifact:
    - `rust/target/vhdl_stimuli_quality_gate/work/vhdl_parseability_generation_report.json`
  - focused replay-shadow evidence at `PGEN_VHDL_STIMULI_QUALITY_COUNT=2` currently shows:
    - authoritative replay target debt: `254 -> 0`
    - parser-backed replay shadow: `requested_total=550`, `accepted_total=109`, `rejected_total=441`, `acceptance_rate_percent=19.82`
- parse-full stage behavior:
  - `auto`: gate builds a temporary `vhdl` parser-registry adapter from generated parser artifact and runs parse-full when available,
  - `0`: disabled,
  - `1`: strict required mode (fails gate when adapter unavailable or any sample parse-full rejects).
- aggregate policy default:
  - `sota_exit_gate` now executes this stage as required strict (`run=1`, `strict=1`) via:
    - `PGEN_SOTA_POLICY_RUN_VHDL_STIMULI_QUALITY=1`
    - `PGEN_SOTA_POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT=1`.
- aggregate observability behavior (`sota_exit_gate`):
  - aggregate stage state dir:
    - `rust/target/sota_exit_gate/work/vhdl_stimuli_quality_gate`
  - aggregate output and `summary.txt` include:
    - `vhdl_stimuli_quality_state_dir`
    - `vhdl_stimuli_quality_closed_loop_initial_targets`
    - `vhdl_stimuli_quality_closed_loop_replay_targets`
    - `vhdl_stimuli_quality_closed_loop_parseability_shadow_*`
    - `vhdl_stimuli_quality_parseability_generation_*`
    - `vhdl_stimuli_quality_realistic_corpus_report_json`
- deterministic realistic-corpus stage:
  - contract keys (`vhdl_core_v0_contract.json`):
    - `realistic_corpus.enforce`
    - `realistic_corpus.cases_path`
    - `realistic_corpus.max_parse_full_ms_per_case`
    - `realistic_corpus.max_sample_bytes`
  - runtime controls:
    - `PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE=auto|0|1`
    - `PGEN_VHDL_STIMULI_REALISTIC_CORPUS`
    - `PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MAX_CASES`
  - deterministic report artifact:
    - `rust/target/vhdl_stimuli_quality_gate/work/vhdl_realistic_corpus_report.json`
  - behavior:
    - runs curated VHDL fixtures through parse-full with per-case latency/size budget enforcement,
    - `expect_parse_full_pass=true` => parse-full pass is required,
    - `expect_parse_full_pass=false` => parse-full fail is currently accepted; parse-full pass is counted as improvement telemetry.
  - current deterministic corpus baseline (`vhdl_realistic_corpus_v0.json`, `version: 2`):
    - `14` total cases (`8` expected-pass, `6` expected-fail),
    - pass families include record/package usage, component port-map instantiation, process if/else assignment, configuration declaration, and context declaration,
    - expected-fail families include currently tracked gaps such as labeled `for ... generate`, `wait for <time-unit>`, and `assert ... report ...`.

`sv_syntax_closure_gate` no-regression contract:
- deterministic flow:
  - `EBNF -> JSON -> parser` + one-pass deterministic syntax probe (`coverage/gap` summary).
- contract-enforced invariants:
  - unresolved rule-reference budget (`max_unresolved_rule_references`),
  - entry-rule defined and unique rule-name constraints,
  - reachable/unreachable rule and branch summary caps.
- default contract artifact:
  - `rust/test_data/grammar_quality/systemverilog_syntax_closure_contract.json`

Optional stimuli-module parity-gate tuning:
- `PGEN_STIMULI_MODULE_PARITY_COUNT` (default `16`)
- `PGEN_STIMULI_MODULE_PARITY_GAP_THRESHOLD` (default `1`)
- `PGEN_STIMULI_MODULE_PARITY_MAX_DEPTH` (default `24`)
- `PGEN_STIMULI_MODULE_PARITY_MAX_REPEAT` (default `4`)
- `PGEN_STIMULI_MODULE_PARITY_CONTRACT` (override grammar contract manifest path)

## 11) Embedding API (Rust)

Stable module:
- `rust/src/embedding_api.rs`

Contract docs:
- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`

Local check:
```bash
make -C rust embedding_api_gate
make -C rust nexsim_parser_embedding_contract_gate
make -C rust regex_parser_integration_contract_gate
```

Design goal:
- Expose stable, versioned outcomes for embedders without coupling to internal AST implementation details.

Current stable surfaces:
- Idiomatic Rust `Result` APIs:
  - `parse_annotation_result(...)`
  - `parse_annotation_with_limits_result(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_with_limits_result(...)`
  - `parse_grammar_profile_ast_dump_result(...)`
  - `parse_grammar_profile_ast_dump_with_limits_result(...)`
- Deterministic outcome APIs:
  - `parse_annotation(...)`
  - `parse_annotation_with_limits(...)`
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_with_limits(...)`
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_ast_dump_with_limits(...)`
- Language-neutral named APIs:
  - `parse_annotation_named(...)`
  - `parse_annotation_named_with_limits(...)`
  - `parse_grammar_profile_named(...)`
  - `parse_grammar_profile_named_with_limits(...)`
  - `parse_grammar_profile_ast_dump_named(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits(...)`
- Parser profile embedding metadata:
  - `parser_embedding_api_contract()`
  - includes `profile_matrix` (`grammar -> supported profiles`)
  - includes integration invariants:
    - `input_ownership_model=borrowed_str`
    - `parse_session_model=stateless_per_call`
    - `zero_copy_input_boundary=true`
    - `stable_diagnostic_codes=[E_BACKEND_UNAVAILABLE,E_INPUT_TOO_LARGE,E_INVALID_ARGUMENT,E_INVALID_LIMITS,E_PARSE_FAILURE,E_UNSUPPORTED_PROFILE]`

Current parser profiles:
- `systemverilog`: `sv_2017`, `sv_2023`
- `vhdl`: `vhdl_1076_2019`
- `regex`: `regex_default`

Host-oriented convenience parser entry points:
- `parse_systemverilog_2017(...)`
- `parse_systemverilog_2023(...)`
- `parse_vhdl_1076_2019(...)`
- `parse_regex_default(...)`
- each has corresponding `*_with_limits`, `*_result`, and `*_with_limits_result` variants.

Host-oriented convenience AST dump entry points:
- `parse_systemverilog_2017_ast_dump(...)`
- `parse_systemverilog_2023_ast_dump(...)`
- `parse_vhdl_1076_2019_ast_dump(...)`
- `parse_regex_default_ast_dump(...)`
- each has corresponding `*_ast_dump_with_limits(...)` variants.

Operator note:
- the public embedding API now exposes regex through `regex_default`, even though `nexsim_parser_embedding_contract_gate` remains SV/VHDL-specific.
- downstream projects should start from the root parser-family integration docs rather than only the generic embedding contract:
  - `PGEN_PARSER_INTEGRATION_CONTRACTS.md`
  - `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
  - `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`

Deterministic integration behavior:
- grammar/profile mismatch returns `E_UNSUPPORTED_PROFILE`.
- missing generated backend returns `E_BACKEND_UNAVAILABLE`.
- invalid family/backend/grammar/profile names in named APIs return `E_INVALID_ARGUMENT`.
- per-call bounded input limits enforced via `ParseLimits`.
- dedicated parser-profile contract gate executes in both build modes:
  - `cargo test --lib parser_embedding_`
  - `cargo test --features generated_parsers --lib parser_embedding_`

AST dump integration behavior:
- AST dump controls use `AstDumpOptions`:
  - `pretty` (compact vs pretty JSON)
  - `max_ast_bytes` (optional bounded payload size)
- AST dump payload uses `AstDumpPayload`:
  - `dump_json`
  - `truncated`
  - `full_bytes`
  - `emitted_bytes`
- when bounded payload exceeds `max_ast_bytes`, deterministic truncation envelope is returned in `dump_json` (`kind=pgen_ast_dump_truncation`, `dump_kind=parser_return_ast`).

### Regex Parser Flavor

Public contract identity:
- parser family:
  - `regex`
- stable profile:
  - `regex_default`
- parser release version:
  - `1.1.1`
- integration contract version:
  - `1.1.1`
- embedding API baseline:
  - `1.2.0`
- AST-dump schema version:
  - `1`

Current measured operational baseline:
- family status:
  - `Done`
- parser-backed family proof:
  - `parseability_attempts_total=1554`
  - `parseability_accepted_total=1554`
  - `parseability_rejected_total=0`
  - `parseability_parser_rejections_total=0`
  - `initial_targets=355`
  - `resolved_targets=355`
  - `final_targets=0`
- broader checked-in corpus proof:
  - `cases_executed=44`
  - `parse_pass_total=44`
  - `parse_fail_total=0`
- formal closure:
  - green
- family closure counts:
  - `8/8/0`

Accepted syntax families in the current published flavor:
- empty regex
- raw regex bodies rather than host-language delimiter wrappers
- literal concatenation
- whole-pattern recursion via `(?R)`
- alternation with `|`
- dot atom `.`
- whitespace as a literal atom
- bare literal punctuation outside char classes includes:
  - `!`
  - `"`
  - `#`
  - `%`
  - `&`
  - `'`
  - `,`
  - `-`
  - `/`
  - `:`
  - `;`
  - `<`
  - `=`
  - `>`
  - `@`
  - `_`
  - backtick
  - `~`
- anchors:
  - `^`
  - `$`
  - `\A`
  - `\Z`
  - `\z`
  - `\b`
  - `\B`
  - `\G`
- quantifiers:
  - `*`
  - `+`
  - `?`
  - counted forms such as `{3}`, `{2,}`, `{2,4}`, `{,4}`, and `{,}`
  - lazy suffix `?`
  - possessive suffix `+`
  - final-atom binding across literal runs, so `ab+` is transported as literal `a` followed by quantified `b`
- escapes:
  - simple escaped characters such as `\n`, `\t`, `\\`
  - hexadecimal forms such as `\xFF` and `\x{FFFF}`
  - Unicode forms such as `\u{FFFF}`
  - octal escapes
  - control escapes such as `\cA`
  - Unicode property escapes `\p{...}` and `\P{...}`
- backreferences:
  - numeric forms such as `\1`
  - named forms such as `\k<name>`, `\k'name'`, and `\k{name}`
  - numeric forms are preserved as `backreference` constructs rather than generic escapes in the AST dump
- character classes:
  - simple classes such as `[abc]`
  - negated classes such as `[^abc]`
  - ranges such as `[a-z]`
  - mixed ranges such as `[a-zA-Z0-9]`
  - class escapes
  - POSIX classes such as `[[:digit:]]`
  - negated POSIX classes such as `[[:^alnum:]]`
- groups:
  - capturing groups `(abc)`
  - noncapturing groups `(?:abc)`
  - named groups `(?<name>abc)` and `(?'name'abc)`
  - atomic groups `(?>abc)`
- lookarounds:
  - positive lookahead
  - negative lookahead
  - positive lookbehind
  - negative lookbehind
- inline modifier forms
- scoped inline modifier forms
- conditional forms:
  - condition may be a lookaround, a bare name, an explicit name reference, digits, signed digits, or a recursion condition
  - explicit false branches are preserved separately, so `(?(1)a|b)` transports `a` and `b` as distinct yes/no branches
- embedded code-block forms:
  - plain `(?{...})`
  - language-tagged `(?{lua: ...})`
  - language-tagged `(?{js: ...})`
  - language-tagged `(?{javascript: ...})`

Representative accepted examples:
- ``
- `a|b`
- `ab+`
- `https?://[^\\s]+`
- `^abc$`
- `\\bword\\b`
- `(?R)`
- `(foo|bar)+`
- `(?<name>[a-z]+)`
- `[[:^alnum:]]+`
- `(a)\\1`
- `(?(1)a|b)`
- `(?<A>foo)-\\k{A}`
- `(?<A>a)?(?(A)b|c)`
- `a{,4}`
- `(?>ab|cd)`
- `(?=abc)abc`
- `(?{lua: return x + 1})`
- `(?{javascript:return x + 1;})`

Regex code-block handling:
- embedded code blocks are parsed structurally
- plain `(?{...})` is preserved as opaque generic payload
- language tags `lua`, `js`, and `javascript` are preserved as opaque source-body payloads
- balanced braces inside code blocks are handled explicitly
- double-quoted and single-quoted strings inside code blocks are handled explicitly
- escaped characters inside code blocks are handled explicitly
- this parser contract is about acceptance, AST transport, and diagnostics
- this parser contract does not yet claim arbitrary valid Lua or JavaScript payload acceptance beyond those published structural forms
- JavaScript comment/template-literal shielding and Lua long-bracket shielding are the next explicit parser-layer follow-up, not a current published guarantee
- this parser contract does not yet publish `native` or `wasm` tagged code blocks
- it is not a promise about runtime execution semantics for those code blocks
- dedicated structural proof for this slice lives at:
  - `make -C rust regex_embedded_code_block_contract_gate`

Diagnostics and AST behavior:
- stable diagnostic codes:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`
- generated regex parse failures are expected to expose:
  - `location.byte_offset`
  - `location.line`
  - `location.column`
- regex AST dumps are stable as JSON transport plus schema version `1`
- the stable AST node envelope is:
  - `rule_name`
  - `span.start`
  - `span.end`
  - `content`
- parser release `1.1.1` specifically fixes several accepted-tree semantic transport bugs while keeping that JSON schema version stable:
  - `(?R)` now appears as `subroutine_call` / `subroutine_target`, not `inline_modifiers`
  - `\1` now appears as `backreference`, not `escape`
  - `(?(1)a|b)` now emits separate `yes_branch` and `no_branch`
  - `ab+` now emits separate pieces for `a` and `b+` instead of a single `(ab)+`-style piece
- the generated regex host path now also applies a compile-style validation layer after parse success, so several obvious compile-invalid forms are rejected deterministically instead of surfacing as false accepts:
  - unsupported `\i`
  - counted-quantifier inversions such as `{5,4}`
  - counted-quantifier bounds above `65535`
  - forbidden character-class escapes such as `[\B]`, `[\R]`, `[\X]`
  - descending character-class ranges such as `[z-a]`
  - quantified anchors such as `^+`
  - variable-length lookbehind such as `(?<=a+)b`

Current non-promises and boundaries:
- this is not a blanket claim of compatibility with every PCRE, PCRE2, RE2, Oniguruma, JavaScript, .NET, or Rust-regex dialect feature
- this is not a host-language regex literal parser for wrapper forms such as `/pattern/flags`
- this is not a runtime contract for executing embedded code blocks
- this is not a promise of stable internal Rust parser or AST node types
- downstream AST consumers should pin parser release version and rerun their own AST compatibility corpus on upgrade

Downstream operational recommendation:
- if another project needs the regex parser, start with `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- use `parser_embedding_api_contract()` at startup to record:
  - `supports_regex_generated_backend`
  - `regex_parser_release_version`
  - `regex_integration_contract_version`
  - `regex_ast_dump_schema_version`
- if something misbehaves, collect the structured outcome, AST dump when relevant, and trace bundle described in `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

### Regex External Corpus Hardening

Current role:
- the checked-in regex family row is still `Done`
- future external hardening should now use `regex_corpus_bundle/` as the maintained acquisition/inventory starter rather than ad hoc corpus hunting
- this lane does not reopen the closed regex family row unless normalized/oracle-backed evidence is intentionally promoted into the tracked regex closure contract

Corpus doctrine:
- canonical syntax source:
  - PCRE2 upstream `testdata/testinput*` and related files
- secondary PCRE2-relevant source:
  - PHP `ext/pcre/tests`
- quarantine only:
  - non-PCRE2 engines or random regex collections

Operational entrypoints:
- preflight bundle contract:
  - `make -C rust regex_corpus_bundle_contract_gate`
- text-safe acceptance slice:
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
- compile-oracle slice:
  - `make -C rust regex_pcre2_compile_oracle_gate`
- bundle overview:
  - `regex_corpus_bundle/README.md`
- detailed acquisition/normalization plan:
  - `regex_corpus_bundle/docs/regex_corpus_plan.md`
- fetch pinned upstream inventories:
  - `python3 regex_corpus_bundle/scripts/fetch_regex_corpora.py --all`

Important interpretation:
- PCRE2 upstream is raw syntax truth for this lane
- PHP corpus is useful because it is PCRE2-backed, but it must be wrapper-normalized before it counts as raw parser truth
- the text-safe gate is useful for widening accepted-syntax evidence, but it is not a correctness oracle because PCRE2 testdata contains both valid and intentionally invalid patterns
- the compile-oracle gate is the first external-corpus lane that actually measures expected compile outcomes against PCRE2 source truth
- the latest hardening slice is not just more measurement: the generated regex host path now also enforces a small compile-style validation layer for obvious invalid constructs that the raw grammar would otherwise over-accept
- current tracked compile-oracle baseline:
  - `cases_executed=2195`
  - `expected_parse_ok_total=1613`
  - `expected_parse_fail_total=582`
  - `parse_expectation_match_total=1668`
  - `parse_expectation_mismatch_total=527`
  - `false_accept_total=325`
  - `false_reject_total=202`
- the current downstream regex release aligned with that hardening slice is:
  - parser release version `1.1.1`
  - integration contract version `1.1.1`
- the current improvement came from two complementary changes:
  - the grammar now accepts more real PCRE2 surface such as negated POSIX classes, bare-name / signed conditional references, `\k{name}`, and `{,}` counted-quantifier forms
  - the host path now rejects obvious compile-invalid forms such as `\i`, bad counted quantifier bounds, forbidden class escapes like `[\B]`, descending class ranges, quantified anchors, and variable-length lookbehind
- future normalizer work should stay split by source family:
  - `normalize_pcre2_testdata.py`
  - `normalize_pcre2_compile_oracle.py`
  - `normalize_php_pcre_tests.py`

## 12) File and Artifact Map

Sources:
- `grammars/*.ebnf`

LRM conversion workspaces (tracked versioned outputs):
- `docs/systemverilog/2017/`
  - `txt/`, `md/`, `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`
- `docs/systemverilog/2023/`
  - `txt/`, `md/`, `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`
- `docs/vhdl/2019/`
  - `txt/`, `md/`, `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`
- `docs/verilog/2005/`
  - `txt/`, `md/`, `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`

Canonical extracted EBNF snapshots:
- `grammars/systemverilog_2017_lrm_extracted.ebnf`
- `grammars/systemverilog_2023_lrm_extracted.ebnf`
- `grammars/verilog_2005_lrm_extracted.ebnf`

LRM conversion tooling (adapted scripts under `tools/`):
- `tools/split_sections.py`
- `tools/txt_to_md_converter.py`
- `tools/extract_grammar.py`
- `tools/extract_grammar_v2.py`
- `tools/create_clean_grammar.py`
- `tools/ieee_lrm_converter.py`
- workflow notes: `tools/LRM_CONVERSION_WORKFLOW.md`

LRM conversion quick commands:
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf \
  --out-root docs/systemverilog/2017 \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2017" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar

python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2023.pdf \
  --out-root docs/systemverilog/2023 \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2023" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar

python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf \
  --out-root docs/vhdl/2019 \
  --document "VHDL Language Reference Manual" \
  --standard "IEEE 1076-2019" \
  --domain "VHDL" \
  --clause-depth 1 \
  --extract-grammar

python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf \
  --out-root docs/verilog/2005 \
  --document "Verilog Hardware Description Language Reference Manual" \
  --standard "IEEE 1364-2005" \
  --domain "Verilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar
```

Generated parser artifacts:
- `generated/*.json`
- `generated/*_parser.rs`
- `generated/*_stimuli.rs`

Ephemeral/runtime reports:
- `rust/target/differential_harness/*`
- `rust/target/performance_gate/report.json`
- `rust/target/fixed_point_gate/*`
- `rust/target/annotation_robustness_gate/*`
- `rust/target/annotation_stimuli_quality_gate/*`
- `rust/target/ebnf_stimuli_quality_gate/*`
- `rust/target/sv_preprocessor_quality_gate/*`
- `rust/target/ast_dump_contract_gate/*`
- `rust/target/sota_exit_gate/*`

Important:
- Do not treat `rust/target/*` reports as source-of-truth grammar artifacts.
- Source-of-truth generated grammar artifacts are under `generated/`.

## 13) Troubleshooting

### Parseability validation error about generated parsers
Use:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- ...
```

### Differential gate fails
1. Inspect reports under `rust/target/differential_harness`.
2. Determine whether mismatch is expected debt or new regression.
3. If expected debt was intentionally changed, refresh baseline:
```bash
make -C rust SHELL=/bin/bash differential_refresh_baseline
```

### Performance gate fails
1. Inspect `rust/target/performance_gate/report.json`.
2. Check parser failure counts first, then throughput/latency budgets.
3. Adjust thresholds only with evidence and changelog updates.

## 14) Recommended Daily Workflow

1. Regenerate/build the parser path you touched.
2. If Rust or generated Rust changed, run clippy flow:
   - `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`
   - source clippy is strict; generated clippy runs every time and can be made strict with `PGEN_CLIPPY_GENERATED_STRICT=1`.
3. Run focused universal tests for return/semantic.
4. Run annotation contract gate (`annotation_contract_gate`) for annotation-heavy changes.
5. Run strict closed-loop stimuli verification (`annotation_stimuli_quality_gate`) when touching stimuli/coverage/gap logic.
6. Run non-annotation closed-loop verification (`ebnf_stimuli_quality_gate`) when touching generic EBNF frontend/parser/stimuli logic.
7. Run SV preprocessor closed-loop verification (`sv_preprocessor_quality_gate`) when touching `systemverilog_preprocessor.ebnf` or preprocessor-specific generation logic.
8. Run differential regression gate.
9. Run fixed-point gate before merge-sensitive changes.
10. Run performance gate for parser/runtime-impacting changes.
11. Run the aggregate release gate (`sota_exit_gate`) before merge/release cuts.
12. Update `CHANGES.md` and `DEVELOPMENT_NOTES.md` for non-trivial behavior changes.
