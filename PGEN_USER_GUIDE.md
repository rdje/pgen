# PGEN User Guide

Last updated: 2026-02-28

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

## 3) Fast Start

### Build and run core gates
```bash
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash sota_release_policy
make -C rust SHELL=/bin/bash fixed_point_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
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

## 5) `ast_pipeline` CLI Guide

Reference:
```bash
cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- --help
```

Primary modes:

1. Transform JSON
- `ast_pipeline INPUT.json [OUTPUT.json]`

2. Generate parser
- `ast_pipeline INPUT.json --generate-parser --output PARSER.rs`

3. Generate stimuli
- `ast_pipeline INPUT.json --generate-stimuli --count N --seed S --output samples.txt`
- This is the default stimuli flow (in-memory generation, optional newline output artifact).

4. Generate stimuli module
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

5. Preprocess SystemVerilog source
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
- `--recovery-stimuli-mode` (`baseline`, `recovery_biased`, `near_sync_negative`)
- `--validate-parseability`
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
  - if flag is present with no value, default path is `gen_ast.log`.
- `--dump-gen-ast-pretty`
  - pretty JSON formatting mode (requires `--dump-gen-ast`).

Mode contract:
- `--dump-gen-ast` is valid only with generation modes:
  - `--generate-parser`
  - `--generate-stimuli`
  - `--generate-stimuli-module`

Examples:
```bash
# Default dump path (gen_ast.log) while generating parser
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
```

Examples:
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
  --output /tmp/semantic_replay.txt
```

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
- `sota_release_policy` (local utility target)
  - prints the tracked machine policy consumed by `sota_exit_gate`
- `annotation_contract_gate` (local gate target)
  - validator + built-in/shared contracts + semantic leverage + advanced robustness checks
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
- `annotation_stimuli_quality_gate` (local gate target)
  - strict deterministic closed-loop verification for return/semantic annotation grammars:
    - baseline parseability/coverage/gap,
    - gap-priority generation with merged-coverage invariants,
    - target-driven generation summary integrity,
    - final gap no-regression checks
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
  - parseability mode:
    - `auto` (default): enable parseability/shrink checks when parser-registry adapter exists,
    - `1`: require parseability adapter (fail if unavailable),
    - `0`: coverage/gap-only mode.
- `stimuli_module_parity_gate` (local gate target)
  - strict deterministic parity verification between in-memory stimuli generation and generated `*_stimuli.rs` artifacts:
    - same grammar + seed + generation config,
    - identical sample corpus,
    - identical coverage metrics JSON and gap report JSON (canonicalized compare),
    - contract-driven grammar roster from `rust/test_data/grammar_quality/stimuli_module_parity_contract.json`
- `ebnf_frontend_readiness` (local report target)
  - executes `EBNF -> JSON -> parser/stimuli` readiness checks for `ebnf/json/regex` grammars
- `ebnf_frontend_gate` (local strict target)
  - same checks, but fails on any grammar-flow failure
- `ebnf_frontend_dual_run_diff` (local report target)
  - executes Perl-vs-Rust (`generated/ebnf.rs`) frontend differential report for `ebnf/json/regex`
- `ebnf_frontend_dual_run_gate` (local strict target)
  - same dual-run differential checks, but fails on any mismatch/failure
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
  - `parseability` (sample replay through generated parser adapter).
- parseability stage behavior:
  - generated samples are tracked as per-sample files (manifest-driven) to preserve multiline stimuli integrity.
  - when a sample fails full parse replay, the gate deterministically retries with incremented seeds until parseable or retry budget is exhausted.
  - retry budget env:
    - `PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS` (default `50`).
- current seed status:
  - `grammars/systemverilog.ebnf` passes `EBNF -> JSON -> parser -> stimuli`.
  - `grammars/vhdl.ebnf` passes `EBNF -> JSON -> parser -> stimuli` (initial seed baseline).
  - strict HDL gate (`make -C rust SHELL=/bin/bash hdl_frontend_gate`) is now green for both tracked grammars.
  - aggregate policy default now promotes HDL readiness to strict required mode (`PGEN_SOTA_POLICY_REQUIRE_HDL_FRONTEND_STRICT=1`).
- SystemVerilog syntax-closure tracking artifact:
  - `SV_GRAMMAR_COVERAGE_MATRIX.md`
  - contains Annex-A-aligned anchor mapping, grouped per-rule coverage status, and explicit unresolved-reference closure debt for the current seed grammar.
  - current matrix state (2026-02-27): unresolved rule-reference debt in `grammars/systemverilog.ebnf` is zero; remaining closure work is Annex-A breadth and semantic legality.

EBNF frontend dual-run commands:
```bash
make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff
make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate
```

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

SV preprocessor closed-loop command:
```bash
make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
```

SV parser/stimuli preprocess-first closed-loop command:
```bash
make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
```

VHDL parser/stimuli closed-loop command:
```bash
make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate
```

SV syntax-closure burn-down no-regression command:
```bash
make -C rust SHELL=/bin/bash sv_syntax_closure_gate
```

Stimuli-module parity command:
```bash
make -C rust SHELL=/bin/bash stimuli_module_parity_gate
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
- `PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRE_VHDL_STIMULI_QUALITY_STRICT` (`1`/`0`, default from policy file)
- `PGEN_SOTA_ALLOW_INFORMATIONAL_FAILURES` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRED_CHECKS` (space-separated required check override list)
- `PGEN_SOTA_POLICY_FILE` (override machine policy file path)
- `PGEN_SOTA_EXIT_STATE_DIR` (override output state dir)

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
- `PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR` (default `rust/target/sv_preprocessor_quality_gate`)

`sv_preprocessor_quality_gate` trusted-reference differential taxonomy:
- gate emits differential report JSON at:
  - `rust/target/sv_preprocessor_quality_gate/work/systemverilog_preprocessor_differential_report.json`
- runner interface contract (`PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER`):
  - positional args:
    - `$1`: input SV sample file
    - `$2`: output preprocessed SV file
    - `$3`: output diagnostics JSON file
- differential modes:
  - `0`: disabled
  - `auto`: enabled when runner is executable; otherwise report-only skip
  - `1`: strict (fails when runner unavailable or when any non-`match` taxonomy occurs)
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
PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER=/abs/path/to/reference_runner.sh \
make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
```

Optional SV stimuli quality-gate tuning:
- `PGEN_SV_STIMULI_QUALITY_CONTRACT` (default `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`)
- `PGEN_SV_STIMULI_QUALITY_COUNT` (override contract sample count)
- `PGEN_SV_STIMULI_QUALITY_SEED_BASE` (override contract seed base)
- `PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_SV_STIMULI_QUALITY_MODE` (`sv_file`/`sv_snippet`/`sv_pp_file`/`sv_pp_snippet`/`sv_semantic_file`, default from contract)
- `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE` (`0`/`1`, default `0`)
  - when set to `1` and `PGEN_SV_STIMULI_QUALITY_MODE` is unset, gate auto-selects `sv_semantic_file`.
- `PGEN_SV_STIMULI_QUALITY_LRM_PROFILE` (single LRM profile override, for example `2017` or `2023`)
- `PGEN_SV_STIMULI_QUALITY_LRM_PROFILES` (CSV LRM profile matrix override, for example `2017,2023`)
- `PGEN_SV_STIMULI_QUALITY_DECLARED_IDENTIFIER_SUITE` (override declared-identifier deterministic contract corpus path)
- `PGEN_SV_STIMULI_QUALITY_ENFORCE_DECLARED_IDENTIFIER_SUITE` (`0`/`1`, overrides contract enforcement toggle)
- `PGEN_SV_STIMULI_QUALITY_STATE_DIR` (default `rust/target/sv_stimuli_quality_gate`)

Optional VHDL stimuli quality-gate tuning:
- `PGEN_VHDL_STIMULI_QUALITY_CONTRACT` (default `rust/test_data/grammar_quality/vhdl_core_v0_contract.json`)
- `PGEN_VHDL_STIMULI_QUALITY_COUNT` (override contract sample count)
- `PGEN_VHDL_STIMULI_QUALITY_SEED_BASE` (override contract seed base)
- `PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE` (`auto`/`0`/`1`, default `auto`)
- `PGEN_VHDL_STIMULI_QUALITY_STATE_DIR` (default `rust/target/vhdl_stimuli_quality_gate`)

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
    - enforces deterministic pass/fail expected outcomes from a fixed corpus (for example declared-vs-undeclared assignment/use, `for`/`foreach` iterators, event-control and named-port usage).
- per-profile closed loop:
  - deterministic initial-stage replay check:
    - gate reruns initial closed-loop generation with same seed/profile config and asserts deterministic equivalence for:
      - stimuli text artifact
      - coverage JSON
      - gap JSON
      - gap text report
    - summary metric:
      - `closed_loop_initial_replay_determinism_passes`
  - `coverage/gap(initial) -> target-driven replay -> non-increasing target debt check`.
  - preprocess convergence debt extraction on closed-loop corpora:
    - `closed_loop_initial_preprocess_warnings_total`
    - `closed_loop_initial_preprocess_errors_total`
    - `closed_loop_replay_preprocess_warnings_total`
    - `closed_loop_replay_preprocess_errors_total`
  - with `closed_loop.require_non_increasing_target_debt=true`, gate enforces:
    - `replay_targets <= initial_targets`
    - `replay_preprocess_errors <= initial_preprocess_errors` (per profile)
- per-sample deterministic flow:
  - `stimuli_generate -> preprocess -> parse_full(optional) -> semantic_validate_baseline`.
- profile behavior:
  - contract defines supported/required LRM profiles (`2017`, `2023`) for one common `systemverilog.ebnf`,
  - gate executes selected profile set and reports profile-tagged rows in summary output.
- stimuli mode behavior (from `systemverilog_core_v0_contract.json` `stimuli_modes`):
  - `sv_file`:
    - entry rule: `systemverilog_file`,
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
      - `require_package_qualification_resolution=true`
      - `require_width_compatibility_simple=true`
      - `require_context_legality_basic=true`
      - `require_declared_identifiers_before_use=false` (currently held off in this profile while lexical false-positive hardening continues).
  - mode-level recovery steering:
    - optional profile key:
      - `stimuli_modes.profiles.<mode>.recovery_stimuli_mode`
    - allowed values:
      - `baseline`
      - `recovery_biased`
      - `near_sync_negative`
    - gate forwards this to all mode-run stimuli generation calls via `--recovery-stimuli-mode`.
  - mode-level semantic overrides:
    - optional profile key:
      - `stimuli_modes.profiles.<mode>.semantic_overrides.<semantic_baseline_toggle>`
    - overrides are applied after global `semantic_baseline` defaults to compute effective per-mode semantic checks.
    - current contract policy:
      - `sv_file`: `require_port_binding_legality_basic=true`
      - `sv_snippet`: `require_port_binding_legality_basic=false`
      - `sv_pp_file`: `require_port_binding_legality_basic=true`
      - `sv_pp_snippet`: `require_port_binding_legality_basic=false`
      - `sv_semantic_file`:
        - `require_port_binding_legality_basic=true`
        - `require_package_qualification_resolution=true`
        - `require_width_compatibility_simple=true`
        - `require_context_legality_basic=true`
        - `require_declared_identifiers_before_use=false`
- closed-loop contract controls (from `systemverilog_core_v0_contract.json`):
  - `closed_loop.gap_report_threshold`
  - `closed_loop.target_max_attempts`
  - `closed_loop.replay_sample_count`
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
    - current implementation uses structured use-site scanning (assignments, conditions, event controls, named-port actuals), strips quoted strings/directives, ignores member/namespace/macro contexts, and handles additional declaration contexts (ports/imports/for/foreach/instantiation); still not default-enabled in semantic-closure profile.
    - deterministic contract coverage exists in `rust/test_data/grammar_quality/systemverilog_declared_identifier_contract_cases.json`, including explicit `foreach (arr[idx])` iterator declaration handling.
  - optional package qualification/import resolution heuristic (`semantic_baseline.require_package_qualification_resolution`).
  - optional simple packed-width vs literal-width compatibility check (`semantic_baseline.require_width_compatibility_simple`).
    - current implementation covers packed declarations of `logic|reg|wire|bit` and indexed LHS assignment forms.
  - optional basic context legality checks (`semantic_baseline.require_context_legality_basic`):
    - `always_comb` must not contain event controls,
    - `always_ff` must not contain blocking assignments,
    - generate `for` iterators must be declared `genvar`.
- parse-full stage behavior:
  - `auto`: gate builds a temporary `systemverilog` adapter from the generated parser artifact and runs parse-full when available; parse-full rejections are recorded as soft-fail stage entries (gate continues),
  - `0`: disabled,
  - `1`: required and strict (fails gate if adapter is unavailable or if any sample parse-full rejects).

`vhdl_stimuli_quality_gate` closed-loop stage contract:
- deterministic flow:
  - `EBNF -> JSON -> parser -> coverage/gap(initial) -> target-driven replay -> parse_full(optional)`.
- contract controls (from `vhdl_core_v0_contract.json`):
  - `entry_rule`
  - `sample_count`
  - `seed_base`
  - `closed_loop.gap_report_threshold`
  - `closed_loop.target_max_attempts`
  - `closed_loop.replay_sample_count`
  - `closed_loop.require_non_increasing_target_debt`
- parse-full stage behavior:
  - `auto`: gate builds a temporary `vhdl` parser-registry adapter from generated parser artifact and runs parse-full when available,
  - `0`: disabled,
  - `1`: strict required mode (fails gate when adapter unavailable or any sample parse-full rejects).

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
- `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`

Local check:
```bash
make -C rust SHELL=/bin/bash embedding_api_gate
make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate
```

Design goal:
- Expose stable, versioned outcomes for embedders without coupling to internal AST implementation details.

Current stable surfaces:
- Idiomatic Rust `Result` APIs:
  - `parse_annotation_result(...)`
  - `parse_annotation_with_limits_result(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_with_limits_result(...)`
- Deterministic outcome APIs:
  - `parse_annotation(...)`
  - `parse_annotation_with_limits(...)`
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_with_limits(...)`
- Language-neutral named APIs:
  - `parse_annotation_named(...)`
  - `parse_annotation_named_with_limits(...)`
  - `parse_grammar_profile_named(...)`
  - `parse_grammar_profile_named_with_limits(...)`
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

Nexsim-oriented convenience parser entry points:
- `parse_systemverilog_2017(...)`
- `parse_systemverilog_2023(...)`
- `parse_vhdl_1076_2019(...)`
- each has corresponding `*_with_limits`, `*_result`, and `*_with_limits_result` variants.

Deterministic integration behavior:
- grammar/profile mismatch returns `E_UNSUPPORTED_PROFILE`.
- missing generated backend returns `E_BACKEND_UNAVAILABLE`.
- invalid family/backend/grammar/profile names in named APIs return `E_INVALID_ARGUMENT`.
- per-call bounded input limits enforced via `ParseLimits`.
- dedicated parser-profile contract gate executes in both build modes:
  - `cargo test --lib parser_embedding_`
  - `cargo test --features generated_parsers --lib parser_embedding_`

## 12) File and Artifact Map

Sources:
- `grammars/*.ebnf`

LRM conversion workspaces (local, generated artifacts ignored by default):
- `docs/systemverilog/`
  - `txt/`, `md/`, grammar extraction artifacts from IEEE 1800-2023 conversion
- `docs/vhdl/`
  - `txt/`, `md/`, grammar extraction artifacts from IEEE 1076-2019 conversion

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
  --pdf /Users/richarddje/Documents/github/1800-2023.pdf \
  --out-root docs/systemverilog \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2023" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --extract-grammar

python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/ieee-1076-2019.pdf \
  --out-root docs/vhdl \
  --document "VHDL Language Reference Manual" \
  --standard "IEEE 1076-2019" \
  --domain "VHDL" \
  --clause-depth 1 \
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
