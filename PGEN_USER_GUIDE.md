# PGEN User Guide

Last updated: 2026-02-18

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

## 3) Fast Start

### Build and run core gates
```bash
make -C rust SHELL=/bin/bash fixed_point_gate
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

High-value stimuli flags:

- `--entry-rule`
- `--max-depth`
- `--max-repeat`
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

Important:
- Parseability validation currently supports generated parser checks for:
  - `return_annotation`
  - `semantic_annotation`
- Parseability checks require building with generated parsers:
```bash
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline -- ...
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

Bootstrap behavior notes:
- Bootstrap parser is intentionally permissive and has known quirks.
- For implementation-accurate bootstrap behavior, see:
  - `grammars/builtin_return_annotation.ebnf`
  - `rust/src/ast_pipeline/unified_return_ast.rs`

## 8) Semantic Annotation Features

Main grammar: `grammars/semantic_annotation.ebnf`

Typical forms:

- Basic key/value:
  - `@type: "Expression"`
- Precedence and metadata:
  - `@precedence: 5`
  - `@effect: "pure"`
- Structured values:
  - arrays, objects, tuples, sets, maps
- Expression-style values:
  - arithmetic/logical/comparison/conditional/function-call/lambda forms

Bootstrap behavior notes:
- Bootstrap semantic parser is intentionally permissive.
- Full behavior reference:
  - `grammars/builtin_semantic_annotation.ebnf`
  - `rust/src/ast_pipeline/unified_semantic_ast.rs`

Current leverage contract (parser + stimuli):
- Parser codegen leverage:
  - `TransformExpr` is currently applied on regex atom generation paths for the annotated rule.
  - Canonical parse transforms (`str::parse::<T>().unwrap_or(default)`) produce transformed terminal output in generated parser code.
  - Target type parsing is path-aware (for example `std::primitive::i64`).
  - `Raw` semantic annotations do not currently alter parser regex atom behavior.
- Stimuli generation leverage:
  - For regex-token sampling, semantic hints are checked first for the current rule.
  - Typed hints are derived from canonical transform parsing:
    - `parse::<f*>` -> `"1.0"`
    - `parse::<i*>`, `parse::<u*>`, `parse::<isize>`, `parse::<usize>` -> `"1"`
    - `parse::<bool>` -> `"true"`
  - Non-canonical transform expressions do not override regex sampling.
  - Raw quoted semantic payloads (for example `"literal"`) are unquoted and emitted as the sample.
- Gate/test coverage:
  - `make -C rust semantic_usage_gate`
  - This enforces targeted `semantic_usage_*` unit tests in parser codegen and stimuli generator paths.

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

2. Refresh tracked mismatch baseline:
```bash
make -C rust SHELL=/bin/bash differential_refresh_baseline
```

3. Regression-only gate (fail only on NEW mismatches):
```bash
make -C rust SHELL=/bin/bash differential_regression_gate
```

Tracked baselines:
- `rust/test_data/differential_baseline/return_annotation_baseline.json`
- `rust/test_data/differential_baseline/semantic_annotation_baseline.json`

## 10) CI Gates and What They Protect

- `fixed-point-gate`
  - deterministic bootstrap artifact regeneration
- `performance-gate`
  - throughput/latency/failure thresholds
- `differential-regression-gate`
  - no new generated-vs-bootstrap mismatches
- `embedding_api_gate` (local gate target)
  - contract stability for embedding API behavior

Workflow files:
- `.github/workflows/fixed-point-gate.yml`
- `.github/workflows/performance-gate.yml`
- `.github/workflows/differential-regression-gate.yml`

## 11) Embedding API (Rust)

Stable module:
- `rust/src/embedding_api.rs`

Contract docs:
- `rust/docs/EMBEDDING_API_CONTRACT.md`

Local check:
```bash
make -C rust SHELL=/bin/bash embedding_api_gate
```

Design goal:
- Expose stable, versioned outcomes for embedders without coupling to internal AST implementation details.

## 12) File and Artifact Map

Sources:
- `grammars/*.ebnf`

Generated parser artifacts:
- `generated/*.json`
- `generated/*_parser.rs`

Ephemeral/runtime reports:
- `rust/target/differential_harness/*`
- `rust/target/performance_gate/report.json`
- `rust/target/fixed_point_gate/*`

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
2. Run focused universal tests for return/semantic.
3. Run differential regression gate.
4. Run fixed-point gate before merge-sensitive changes.
5. Run performance gate for parser/runtime-impacting changes.
6. Update `CHANGES.md` and `DEVELOPMENT_NOTES.md` for non-trivial behavior changes.
