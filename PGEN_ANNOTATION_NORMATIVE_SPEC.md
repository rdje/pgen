# PGEN Annotation Normative Specification (Living)

Last updated: 2026-02-20

## Purpose
This document defines the normative contract for PGEN return and semantic annotations across bootstrap and generated pipelines.

The goal is to keep annotation behavior stable for embedding users and to make bootstrap behavior explicit (including known quirks used to break chicken-and-egg cycles).

## Scope Layers
PGEN annotation behavior is defined in three layers:

1. Bootstrap parser contract (built-in, intentionally limited/permissive):
   - `grammars/builtin_return_annotation.ebnf`
   - `grammars/builtin_semantic_annotation.ebnf`
   - Runtime implementations:
     - `rust/src/ast_pipeline/unified_return_ast.rs`
     - `rust/src/ast_pipeline/unified_semantic_ast.rs`
2. Full generated grammar contract:
   - `grammars/return_annotation.ebnf`
   - `grammars/semantic_annotation.ebnf`
3. Typed validation contract (generation-time diagnostics):
   - `rust/src/ast_pipeline/annotation_validator.rs`

## Bootstrap Return Annotation Contract
Normative input/output behavior for bootstrap return parsing:

- Optional arrow stripping is only recognized when `->` starts at byte `0` of raw input.
- Leading/trailing whitespace is trimmed only after optional arrow stripping.
- Empty payload after normalization maps to passthrough (`$1` on round-trip).
- Positional refs (`$N`) are supported, including bootstrap acceptance of `$0`.
- Extraction (`::first`, `::last`, `::N`) is supported, with `::0` rejected.
- Spread suffix (`*`) is supported for positional/extraction forms.
- Property/array access forms are supported.
- Known permissive quirks are part of contract:
  - trailing text after positional spread is ignored (`$1*trailing` -> `$1*`)
  - trailing text after array access is ignored (`$1[0]trailing` -> `$1[0]`)
  - extra commas in top-level arrays/objects are ignored
  - duplicate object keys keep the last value
  - leading whitespace before `->` does not trigger arrow normalization

Source contract references:
- `grammars/builtin_return_annotation.ebnf`
- `rust/src/ast_pipeline/unified_return_ast.rs`

## Bootstrap Semantic Annotation Contract
Normative input/output behavior for bootstrap semantic parsing:

- Input is always outer-trimmed.
- Parser never hard-fails in current behavior.
- Classification to `TransformExpr` is marker-based only:
  - contains `::parse::<`
  - and contains `>().unwrap_or(`
- Any other payload (including empty/nonsensical syntax) is accepted as `Raw`.

Source contract references:
- `grammars/builtin_semantic_annotation.ebnf`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`

## Semantic Leverage Contract (Parser + Stimuli)
Normative runtime leverage behavior for semantic annotations:

- Parser generation (`rust/src/ast_pipeline/ast_based_generator.rs`):
  - `TransformExpr` currently steers regex atom code generation for matching rule names.
  - Canonical parse transforms (`str::parse::<T>().unwrap_or(default)`) emit `TransformedTerminal` code paths.
  - Target type parsing is path-aware (for example `std::primitive::i64`).
  - `Raw` semantic annotations do not alter regex atom parser generation behavior.
- Stimuli generation (`rust/src/ast_pipeline/stimuli_generator.rs`):
  - Regex sample generation checks semantic hints before regex-HIR sampling.
  - Current hint mapping is canonical-transform-driven and contract-enforced:
    - parse float targets -> `"1.0"`
    - parse integer/unsigned/isize/usize targets -> `"1"`
    - parse bool targets -> `"true"`
  - Non-canonical transform expressions do not apply typed hint overrides.
  - raw quoted payloads -> unquoted literal output
- Shared canonical-transform parser utility:
  - `rust/src/ast_pipeline/semantic_transform.rs`
  - Used by validator, parser codegen, and stimuli hinting paths.
- Additional semantic steering contract (Phase K):
  - `@branch_policy` is a typed steering directive used in parser/stimuli OR-branch selection:
    - `longest_match` (default),
    - `ordered`,
    - `priority_first`.
  - `@token_class/@charset/@pattern` token steering baseline:
    - directives are typed/validated by the annotation validator,
    - payload forms:
      - `@token_class`: known class family label (for example `identifier`, `int`, `float`, `bool`, `word`, `alnum`, `lower`, `upper`, `whitespace`, `hex`, `binary`, `printable`),
      - `@charset`: character-class payload (for example `A-Za-z_` or `[0-9A-F]`),
      - `@pattern`: non-empty valid regex payload.
    - deterministic precedence:
      - when multiple token steering directives are present, effective regex policy is:
        - `@pattern` > `@charset` > `@token_class`.
    - parser runtime baseline:
      - regex atom matching uses effective steering regex selected by precedence above.
    - stimuli runtime baseline:
      - regex atom sample generation uses the same effective steering regex selected by precedence above.
    - grammar-aware coherence warning:
      - `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM` when valid token steering directives are present but rule AST has no regex atom (steering remains inactive).
    - Tier-4 gate contract:
      - dedicated shared semantic contract slice:
        - `rust/test_data/semantic_annotation/sc04_contract.json`
      - dedicated gate target:
        - `make -C rust sc04_contract_gate`
      - gate includes differential mismatch taxonomy parity checks over the SC-04 suite:
        - allowed mismatch categories are constrained to the differential taxonomy set,
        - category-count totals must match `mismatched_cases`,
        - SC-04 comparable corpus currently requires `mismatched_cases == 0`.
  - `@coverage_target/@critical_path` contract baseline:
    - directives are typed/validated by the annotation validator,
    - payload forms:
      - `@coverage_target`: non-negative integer or boolean payload (`0/1/2/...`, `true/false`),
      - `@critical_path`: boolean payload,
    - coherence warning:
      - `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET` when `@critical_path` is enabled while effective `@coverage_target` is missing or zero,
    - stimuli runtime baseline uses SC-10 hints to steer coverage exploration:
      - OR-branch guidance multipliers consider current rule SC-10 hints and SC-10 hints on referenced rules,
      - gap-report priority scoring adds semantic bonuses for SC-10-tagged rules/branches.
    - parser runtime instrumentation baseline consumes SC-10 hints in generated parsers:
      - successful parses on SC-10-targeted rules emit `CoverageTargetEvent` records,
      - OR rules tag selected branch index in emitted events,
      - parser exposes deterministic accessors and counters:
        - `coverage_target_events()`
        - `take_coverage_target_events()`
        - `coverage_target_event_count()`
        - `coverage_target_rule_hits()`
        - `coverage_target_branch_hits()`
      - instrumentation remains inactive when effective `@coverage_target` weight is zero.
  - `@invalid_case/@negative` contract baseline:
    - directives are typed/validated by the annotation validator,
    - payload forms:
      - `@invalid_case`: boolean payload,
      - `@negative`: boolean payload,
    - payload diagnostics:
      - `W_SEM_INVALID_INVALID_CASE_PAYLOAD`,
      - `W_SEM_INVALID_NEGATIVE_PAYLOAD`,
    - coherence warning:
      - `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE` when `@negative` is enabled while effective `@invalid_case` is missing/false.
    - parser runtime baseline:
      - generated parsers record expected-failure semantic events on rule failure when effective `@invalid_case` is enabled,
      - emitted event type:
        - `NegativeCaseEvent { rule_name, parse_start, failure_position, negative, error_kind }`
      - parser exposes deterministic accessors/counters:
        - `negative_case_events()`
        - `take_negative_case_events()`
        - `negative_case_event_count()`
        - `negative_case_rule_hits()`
    - stimuli runtime baseline:
      - when effective `@invalid_case` is enabled, entry samples are deterministically mutated toward invalid/near-invalid shape,
      - when effective `@negative` is also enabled, deterministic negative-case marker suffix is appended.
  - `@seed_group/@deterministic_group` determinism partition baseline:
    - directives are typed/validated by the annotation validator,
    - payload forms:
      - `@seed_group`: non-empty label using `[A-Za-z0-9_.-]`,
      - `@deterministic_group`: boolean payload or group-label payload.
    - payload diagnostics:
      - `W_SEM_INVALID_SEED_GROUP_PAYLOAD`,
      - `W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD`,
    - coherence warning:
      - `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP` when `@seed_group` is present while effective `@deterministic_group` is missing/false.
    - parser runtime baseline:
      - for OR rules under `@branch_policy: ordered`, effective deterministic partition hints drive stable branch-evaluation offsets before first-success short-circuit,
      - generated parsers expose explicit runtime partition override modes for embedders:
        - `DeterministicPartitionRuntimeMode::AnnotationDriven`
        - `DeterministicPartitionRuntimeMode::ForceEnabled`
        - `DeterministicPartitionRuntimeMode::ForceDisabled`
      - generated parser API surface includes:
        - `deterministic_partition_runtime_mode()`
        - `set_deterministic_partition_runtime_mode(...)`
      - effective partition-enable decision for branch ordering and event emission is computed at runtime from `(annotation, runtime_mode)`:
        - `AnnotationDriven`: honor annotation payload,
        - `ForceEnabled`: enable partition behavior regardless of annotation,
        - `ForceDisabled`: disable partition behavior regardless of annotation,
      - partition offset is deterministic per rule group key and branch count (group-key hash modulo branch count),
      - group key selection:
        - explicit `@seed_group` label when present,
        - else explicit label embedded in `@deterministic_group`,
        - else fallback `rule.<rule_name>`,
      - parser emits typed partition telemetry for effective deterministic-group rules:
        - event type:
          - `DeterministicPartitionEvent { rule_name, parse_start, parse_end, group_key }`
        - accessors/counters:
          - `deterministic_partition_events()`
          - `take_deterministic_partition_events()`
          - `deterministic_partition_event_count()`
          - `deterministic_partition_rule_hits()`
    - stimuli runtime baseline:
      - when effective `@deterministic_group` is enabled and seed is configured, stimuli generation uses deterministic per-group seed partition routing,
      - group key selection:
        - explicit `@seed_group` label when present,
        - else explicit label embedded in `@deterministic_group`,
        - else fallback `rule.<entry_rule>`,
      - partition counters are maintained per group so sample streams are deterministic and interleaving-order independent across groups.
  - `@recover/@sync/@panic_until` are typed/validated contract directives with executable parser runtime recovery baseline:
    - recovery triggers when OR branches all fail and effective `@recover` is truthy,
    - optional scoped recovery budgets are enforced when present:
      - `@recover_budget`: caps successful recoveries per rule per parse run (`0` disables recovery for that rule in the current parse),
      - `@recover_parse_budget`: caps total successful recoveries across all rules in the current parse (`0` disables recovery for that parse run),
      - `@recover_global_budget`: caps total successful recoveries across parser lifetime (`0` disables recovery for that parser instance),
    - parser scans from rule start for nearest configured marker token (`panic_until` preferred over `sync` on same position),
    - parser advances past selected marker (or to EOF fallback when no marker exists),
    - recovery success continues parse flow with recovered empty branch content,
    - if no forward progress is possible, parser still backtracks.
    - parser records structured typed recovery events for successful recoveries and exposes them via:
      - `recovery_events()`
      - `take_recovery_events()`
      - `recovery_event_count()`
      - `recovery_parse_count()`
      - `recovery_global_count()`
    - event marker kinds:
      - `PanicUntil`
      - `Sync`
      - `EofFallback`
  - Stimuli generation baseline for these directives:
    - when OR branch generation exhausts all alternatives and effective `@recover` is truthy,
    - generator emits deterministic marker fallback sample from recovery directives:
      - first non-empty `@panic_until` token, else first non-empty `@sync` token.
    - dedicated recovery-focused generation modes are available through stimuli config/CLI:
      - `baseline`: standard generation behavior (default),
      - `recovery_biased`: generates base sample then injects recovery marker context for recover-enabled entry rules,
      - `near_sync_negative`: emits near-sync negative-case samples by injecting deterministic invalid noise adjacent to selected recovery marker for recover-enabled entry rules.
  - `@constraint/@requires/@implies` contract baseline:
    - directives are typed/validated by the annotation validator,
    - parser runtime baseline is active when `@constraint` is present:
      - `@requires`: each reference must resolve to a non-empty capture value,
      - `@constraint`: relational expression is evaluated against captures/references,
      - `@implies`: antecedent truth implies consequent truth.
    - parser-side reference resolution supports positional (`$1`, `$2.field`) and named dotted paths (`lhs.id`), including `.len` suffix.
    - parser unresolved references or failed relational checks return generated contextual parse errors.
    - relational hints remain inactive when `@constraint` is missing (validator still emits `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`).
    - stimuli runtime baseline is active for root sequence synthesis when `@constraint` is present:
      - sequence generation retries until relational contract checks pass or attempt budget is exhausted,
      - `@requires`, `@constraint`, and `@implies` use the same evaluator contract surface as parser baseline,
      - stimuli reference support covers positional and named references (including dotted nested paths and optional `.len`),
      - nested path traversal in stimuli is resolved over:
        - structured capture values (for example JSON-like object payloads emitted by grammar rules),
        - non-structured object-like capture values parsed from `=/:` key-value pairs with wrapper-aware delimiter handling,
      - on retry exhaustion, stimuli emits structured diagnostics including relational/generation failure counts, ranked top violation reasons, and a `likely_unsatisfiable` signal,
      - dotted key materialization is supported for non-structured captures (for example `meta.id=AA` -> nested `meta.id` path resolution).

## Typed Annotation Validator Contract
Validator diagnostics are part of normative generation-time behavior.

Current stable diagnostic codes include:

- Return:
  - `E_RET_POS_ZERO`
  - `E_RET_POS_OUT_OF_RANGE`
  - `E_RET_EMPTY_PROPERTY`
  - `E_RET_EMPTY_OBJECT_KEY`
  - `W_RET_UNPARSED`
  - `W_RET_LARGE_EXTRACTION_INDEX`
  - `W_RET_SPREAD_PASSTHROUGH`
  - `W_RET_RULE_NOT_FOUND`
  - `W_RET_BRANCH_INDEX_OOB`
  - `W_RET_BRANCH_NOT_SEQUENCE`
  - `W_RET_POS_RULE_BOUND`
- Semantic:
  - `W_SEM_MARKER_IN_RAW`
  - `W_SEM_NON_CANONICAL_TRANSFORM`
  - `E_SEM_EMPTY_COMPONENT`
  - `W_SEM_UNKNOWN_TARGET_TYPE`
  - `W_SEM_DEFAULT_TYPE_MISMATCH`
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
  - `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`
  - `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE`
  - `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP`
  - `W_SEM_TOKEN_STEERING_PRECEDENCE`
  - `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`
  - `W_SEM_RECOVER_BUDGET_WITHOUT_RECOVER`
  - `W_SEM_RECOVER_PARSE_BUDGET_WITHOUT_RECOVER`
  - `W_SEM_RECOVER_GLOBAL_BUDGET_WITHOUT_RECOVER`
  - `W_SEM_RECOVERY_HINT_WITHOUT_RECOVER`
  - `W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`

Strict mode behavior:
- Strict mode warning promotion is policy-controlled, not blanket:
  - `PGEN_STRICT_SEMANTIC_WARNING_CODES=<comma-separated-codes|all|none>`
  - `all` (or `*`) promotes all semantic warning-class diagnostics to errors,
  - `none` disables strict warning promotions while keeping strict mode enabled for existing error-class checks.
- Strict default profile (when strict mode is enabled and no explicit warning policy is provided):
  - `W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`
  - `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`
- CI/fixed-point paths are expected to run with strict validation enabled.

Source contract reference:
- `rust/src/ast_pipeline/annotation_validator.rs`

## Executable Conformance
Normative contract checks are executable, not only documented:

- Bootstrap return contract suite:
  - `rust/test_data/return_annotation/builtin_contract.json`
- Bootstrap semantic contract suite:
  - `rust/test_data/semantic_annotation/builtin_contract.json`
- Shared bootstrap/generated return contract suite:
  - `rust/test_data/return_annotation/normative_shared_contract.json`
- Shared bootstrap/generated semantic contract suite:
  - `rust/test_data/semantic_annotation/normative_shared_contract.json`
- SC-04 shared semantic contract suite:
  - `rust/test_data/semantic_annotation/sc04_contract.json`
- Semantic leverage usage suite:
  - parser/stimuli unit tests prefixed `semantic_usage_`
- Gate target:
  - `make -C rust annotation_contract_gate`
  - `make -C rust annotation_shared_contract_gate`
  - `make -C rust sc04_contract_gate`
  - `make -C rust semantic_usage_gate`

The gate runs:
- typed validator unit coverage
- bootstrap return contract suite
- bootstrap semantic contract suite
- shared return contract suite (bootstrap + generated)
- shared semantic contract suite (bootstrap + generated)
- SC-04 semantic contract slice + differential taxonomy parity check
- semantic leverage unit contract suite (parser + stimuli)

## Maintenance Rules
When annotation behavior changes intentionally:

1. Update code first.
2. Update corresponding built-in EBNF (`grammars/builtin_*.ebnf`) when bootstrap behavior changed.
3. Update this normative spec.
4. Update contract suites under `rust/test_data/*/builtin_contract.json` and `rust/test_data/*/normative_shared_contract.json`.
5. Keep generated artifacts under `generated/` out of manual edits (they are regenerated from EBNF).
