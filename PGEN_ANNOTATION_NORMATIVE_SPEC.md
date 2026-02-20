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
  - `@recover/@sync/@panic_until` are typed/validated contract directives with executable parser runtime recovery baseline:
    - recovery triggers when OR branches all fail and effective `@recover` is truthy,
    - parser scans from rule start for nearest configured marker token (`panic_until` preferred over `sync` on same position),
    - parser advances past selected marker (or to EOF fallback when no marker exists),
    - recovery success continues parse flow with recovered empty branch content,
    - if no forward progress is possible, parser still backtracks.
  - Stimuli generation baseline for these directives:
    - when OR branch generation exhausts all alternatives and effective `@recover` is truthy,
    - generator emits deterministic marker fallback sample from recovery directives:
      - first non-empty `@panic_until` token, else first non-empty `@sync` token.

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
  - `W_SEM_INVALID_SYNC_PAYLOAD`
  - `W_SEM_INVALID_PANIC_UNTIL_PAYLOAD`
  - `W_SEM_RECOVERY_HINT_WITHOUT_RECOVER`

Strict mode behavior:
- Semantic warning-class checks are promoted to errors when strict mode is enabled.
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
- Semantic leverage usage suite:
  - parser/stimuli unit tests prefixed `semantic_usage_`
- Gate target:
  - `make -C rust annotation_contract_gate`
  - `make -C rust annotation_shared_contract_gate`
  - `make -C rust semantic_usage_gate`

The gate runs:
- typed validator unit coverage
- bootstrap return contract suite
- bootstrap semantic contract suite
- shared return contract suite (bootstrap + generated)
- shared semantic contract suite (bootstrap + generated)
- semantic leverage unit contract suite (parser + stimuli)

## Maintenance Rules
When annotation behavior changes intentionally:

1. Update code first.
2. Update corresponding built-in EBNF (`grammars/builtin_*.ebnf`) when bootstrap behavior changed.
3. Update this normative spec.
4. Update contract suites under `rust/test_data/*/builtin_contract.json` and `rust/test_data/*/normative_shared_contract.json`.
5. Keep generated artifacts under `generated/` out of manual edits (they are regenerated from EBNF).
