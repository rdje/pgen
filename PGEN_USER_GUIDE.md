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

#### D) Relational constraint contract baseline (`@constraint`, `@requires`, `@implies`)
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
  - stimuli reference support covers positional and named nested paths (for example `$1.id`, `lhs.id`, `$3.id.len`) over structured capture payloads, with optional `.len`.
- Coherence rule:
  - `@requires`/`@implies` without `@constraint` triggers `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`.

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
  - dedicated recovery-focused negative-case generation modes remain follow-on work.

### 8.9 Gate/Test Coverage for Semantic Steering
- `make -C rust semantic_usage_gate`
- Included in:
  - `make -C rust annotation_contract_gate`
- Coverage includes parser + stimuli semantic usage tests (`semantic_usage_*`), including value-domain steering, directive routing, parser recovery-hook codegen, and stimuli recovery-fallback regressions.

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
- Strict mode promotes these warning-class diagnostics to errors, same as other semantic contracts.
- Runtime parser enforcement activates only when `@constraint` is present:
  - `@requires` references must resolve and be non-empty.
  - `@constraint` expression is evaluated against capture/reference values.
  - `@implies` is enforced as antecedent truth implies consequent truth.
- Runtime stimuli enforcement activates only when `@constraint` is present on root-sequence rules:
  - generation retries until relational checks pass (or attempt budget is exhausted),
  - same `@requires/@constraint/@implies` checks gate sample acceptance.
- Reference resolution supports positional (`$1`, `$2.field`) and named dotted references (`lhs.id`) including `.len` suffix (for example `$1.len >= 1`).
- Stimuli reference support includes nested named/positional paths (for example `lhs.id`, `$1.id`, `$3.id.len`) when referenced capture values are structured (JSON-like object payloads).
- Broader non-structured nested extraction heuristics remain follow-on hardening.
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
