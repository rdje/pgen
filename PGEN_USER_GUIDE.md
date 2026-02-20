# PGEN User Guide

Last updated: 2026-02-20

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
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash sota_release_policy
make -C rust SHELL=/bin/bash fixed_point_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
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
- Directive routing is name-aware:
  - transform steering is only active for directive `transform`,
  - raw literal hint steering is only active for literal/sample directive family in named mode.

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

#### C) Value-domain steering (`@enum`, `@range`, `@len`, `@regex`)
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
  - `W_SEM_RECOVERY_HINT_WITHOUT_RECOVER` (`@sync`/`@panic_until` present while `@recover` is not enabled)
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

#### Example: Recovery hints contract (current stage)
```ebnf
stmt = declaration | assignment ;
@recover: true
@sync: [";", "end"]
@panic_until: ["}"]
```
- Validator enforces typed payloads and contract coherence.
- Current generated parser/stimuli stage:
  - branch policy is active,
  - recovery hints are accepted/validated and surfaced for staged integration,
  - full panic/sync runtime recovery engine is not yet enabled.

### 8.9 Gate/Test Coverage for Semantic Steering
- `make -C rust semantic_usage_gate`
- Included in:
  - `make -C rust annotation_contract_gate`
- Coverage includes parser + stimuli semantic usage tests (`semantic_usage_*`), including value-domain steering and directive routing regressions.

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
  - one-shot release-grade aggregate check for fixed-point, annotation, differential, performance, embedding, and EBNF readiness reporting
- `sota_release_policy` (local utility target)
  - prints the tracked machine policy consumed by `sota_exit_gate`
- `annotation_contract_gate` (local gate target)
  - validator + built-in/shared contracts + semantic leverage + advanced robustness checks
- `annotation_robustness_gate` (local gate target)
  - advanced return/semantic suites in bootstrap/generated modes + generated parseability/coverage/gap checks
- `ebnf_frontend_readiness` (local report target)
  - executes `EBNF -> JSON -> parser/stimuli` readiness checks for `ebnf/json/regex` grammars
- `ebnf_frontend_gate` (local strict target)
  - same checks, but fails on any grammar-flow failure
- `performance-gate`
  - throughput/latency/failure thresholds
- `differential-regression-gate`
  - no new generated-vs-bootstrap mismatches
- `return_parity_gate` (local gate target)
  - zero return mismatches on expectation-aligned (comparable) differential corpus
- `embedding_api_gate` (local gate target)
  - contract stability for embedding API behavior

Workflow files:
- `.github/workflows/fixed-point-gate.yml`
- `.github/workflows/performance-gate.yml`
- `.github/workflows/differential-regression-gate.yml`
- `.github/workflows/sota-exit-gate.yml`

EBNF frontend readiness commands:
```bash
make -C rust SHELL=/bin/bash ebnf_frontend_readiness
make -C rust SHELL=/bin/bash ebnf_frontend_gate
```

Annotation contract/robustness commands:
```bash
make -C rust SHELL=/bin/bash annotation_robustness_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
make -C rust SHELL=/bin/bash return_parity_gate
```

Aggregate SOTA gate command:
```bash
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash sota_release_policy
```

Aggregate gate tuning:
- `PGEN_SOTA_RUN_EBNF_READINESS` (`1`/`0`, default `1`)
- `PGEN_SOTA_REQUIRE_EBNF_STRICT` (`1`/`0`, default `0`)
- `PGEN_SOTA_ALLOW_INFORMATIONAL_FAILURES` (`1`/`0`, default from policy file)
- `PGEN_SOTA_REQUIRED_CHECKS` (space-separated required check override list)
- `PGEN_SOTA_POLICY_FILE` (override machine policy file path)
- `PGEN_SOTA_EXIT_STATE_DIR` (override output state dir)

Release policy references:
- machine policy: `rust/config/sota_exit_policy.env`
- release checklist/policy doc: `PGEN_RELEASE_POLICY.md`

Optional robustness-gate tuning:
- `PGEN_ANNOTATION_ROBUSTNESS_COUNT` (default `32`)
- `PGEN_ANNOTATION_ROBUSTNESS_RETURN_SEED` (default `4242`)
- `PGEN_ANNOTATION_ROBUSTNESS_SEMANTIC_SEED` (default `4343`)

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
- `rust/target/annotation_robustness_gate/*`
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
2. Run focused universal tests for return/semantic.
3. Run annotation contract gate (`annotation_contract_gate`) for annotation-heavy changes.
4. Run differential regression gate.
5. Run fixed-point gate before merge-sensitive changes.
6. Run performance gate for parser/runtime-impacting changes.
7. Run the aggregate release gate (`sota_exit_gate`) before merge/release cuts.
8. Update `CHANGES.md` and `DEVELOPMENT_NOTES.md` for non-trivial behavior changes.
