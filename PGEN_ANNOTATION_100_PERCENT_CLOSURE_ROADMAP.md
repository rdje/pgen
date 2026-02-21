# PGEN Annotation 100% Closure Roadmap (Living)

Last updated: 2026-02-21

## Non-Negotiable Contract
PGEN must support annotation behavior with zero functional gaps against:
- `grammars/return_annotation.ebnf`
- `grammars/semantic_annotation.ebnf`

No silent degradation is acceptable for grammar-conforming inputs.

## Binding Constraints (Must Not Be Broken)
1. Built-in return parser coverage binding:
- The built-in return annotation parser must support every return-annotation construct used by the full return grammar corpus (`grammars/return_annotation.ebnf`).

2. Built-in semantic parser coverage binding:
- The built-in semantic annotation parser must support every semantic-annotation construct used by the full semantic grammar corpus (`grammars/semantic_annotation.ebnf`).

3. Built-in grammar/implementation lockstep binding:
- `grammars/builtin_return_annotation.ebnf` and `grammars/builtin_semantic_annotation.ebnf` are executable compatibility specs for built-in parser behavior.
- Any parser behavior change requires synchronized updates to built-in EBNF specs and contract tests in the same change.

4. Tooling quality binding:
- If any binding above regresses, CI must fail.
- No "best effort" or "partially supported" interpretation is allowed for covered construct sets.

## Normative Sources of Truth
- Return syntax and shape contract: `grammars/return_annotation.ebnf`
- Semantic syntax and shape contract: `grammars/semantic_annotation.ebnf`
- Bootstrap parser compatibility contract: `grammars/builtin_return_annotation.ebnf`, `grammars/builtin_semantic_annotation.ebnf`
- Runtime validator/codegen/stimuli contract: `rust/src/ast_pipeline/*`

## 100% Completion Criteria (Objective and Verifiable)
The roadmap is complete only when all conditions below are true:

1. Parse Completeness
- Every rule and production alternative in both EBNFs has executable positive coverage.
- Every guarded boundary has executable negative coverage.
- Grammar-conforming inputs parse successfully in non-bootstrap mode without fallback parsing paths.

2. Typed AST Completeness
- Every construct in both EBNFs maps to a typed AST variant (no generic raw fallback for conforming inputs).
- Parse tree to typed AST conversion is deterministic and round-trippable by canonical unparser rules.

3. Runtime Semantics Completeness
- Return annotation constructs produce correct runtime AST transformations in generated parsers.
- Semantic annotation constructs that are policy-only are still fully parsed/validated/observable, and all steering-capable constructs are exercised in parser and stimuli runtime.

4. Differential and Parity Closure
- Comparable bootstrap/generated annotation corpora have zero mismatches.
- Any non-comparable bootstrap-only behavior is explicitly scoped to bootstrap contract suites and excluded from non-bootstrap correctness claims.

5. Gate-Enforced Proof
- Required CI gates fail on any regression of parser acceptance, typed AST mapping, runtime semantics, or deterministic output.
- No release can pass without the annotation 100% gate.
- Required CI gates fail if built-in parser coverage drifts below full-grammar-used construct sets.

6. Test Quality Bar (Uncompromising)
- Every supported construct has:
  - positive parse tests,
  - negative boundary tests,
  - typed AST shape tests,
  - runtime semantics tests (parser and/or stimuli as applicable),
  - deterministic round-trip/canonicalization tests.
- No placeholder behaviors are allowed for grammar-conforming annotation inputs.
- No silent fallback is allowed in non-bootstrap mode for grammar-conforming annotation inputs.

## Current Baseline Snapshot (2026-02-21)
Verified facts from the current Rust AST pipeline:

- Non-bootstrap return annotation entry path now requires generated parser success and no longer falls back to `parse_bootstrap`:
  - `rust/src/ast_pipeline/mod.rs` (`parse_return_annotation_ast`)
  - `rust/src/ast_pipeline/unified_return_ast.rs` (`parse_generated_return_annotation`)
- Return typed-AST conversion in non-bootstrap mode now uses structural generated parse-tree mapping (no span-reparse shortcut) across return construct families, with generated-vs-bootstrap structural corpus parity tests including zero/signed-zero positional semantics and extraction-index normalization.
- Semantic annotation typed parsing in pipeline is still bootstrap marker/raw-oriented:
  - `rust/src/ast_pipeline/unified_semantic_ast.rs`
  - `rust/src/ast_pipeline/mod.rs` (`parse_semantic_annotation_entry`)
- Generated return parser round-trip now emits canonical typed unparse output (identity shortcut removed):
  - `rust/src/bin/test_runner.rs`

Implication: existing gates are strong, but they do not yet prove full typed-AST/runtime closure for every construct defined in the two full EBNFs.

## Workstream A: Return Annotation Closure (RA)

### RA Scope Inventory (from `grammars/return_annotation.ebnf`)
- Entry and arrow forms: `return_annotation`, `arrow`
- Expression dispatch: `expression`, `primary_expression`, `parenthesized`
- Positional and literals: `positional_reference`, `string_literal`, `number_literal`, `boolean_literal`, `identifier`
- Postfix and extraction: `extraction_expression`, `extraction_target`, `spread_expression`, `spreadable_expression`, `spread_suffix`
- Access chains: `property_access_expression`, `array_access_expression`, `accessor_base`
- Structured forms: `object_literal`, `object_properties`, `object_property`, `property_key`, `array_literal`, `array_elements`, `array_element`

### RA Milestones

#### RA-01 Parse Tree to Typed AST Closure
- Implement generated-parser-backed parse tree to `UnifiedReturnAST` conversion for all return grammar constructs.
- In non-bootstrap mode, remove bootstrap parser as typed AST source for conforming return inputs.
- Keep bootstrap parser only for explicit bootstrap mode.
Status (2026-02-21):
- In progress.
- Non-bootstrap return entry path now enforces generated parse success and no bootstrap fallback.
- Structural parse-tree-to-typed-AST mapping is now implemented for return construct families exercised by contract/parity suites.
- Remaining closure item: objective RA-01 proof closure via explicit construct/alternative manifest coverage gating (`PX-01`) to prove 100% mapping coverage.

Exit criteria:
- 0 conforming return samples use bootstrap fallback in non-bootstrap mode.
- 100% of return grammar construct corpus maps to typed `UnifiedReturnAST`.

#### RA-02 Runtime Transform Closure
- Complete `AstReturnTransformer` behavior for all return AST nodes and combinations:
  - positional and nested sequence access
  - spread behavior in array/object contexts
  - extraction targets (`index`, `first`, `last`) with and without spread
  - chained property and array access with expression index semantics
  - identifier expression handling parity with grammar contract
  - single-quoted and double-quoted string support parity
- Remove placeholder or defaulting behavior that changes meaning of valid expressions.

Exit criteria:
- No placeholder outputs for conforming inputs.
- Runtime expected output corpus passes for every return construct family.

#### RA-03 Canonical Round-Trip Closure
- Implement canonical unparse from typed return AST for generated parser path.
- Replace generated round-trip identity shortcut in test runner with canonical typed round-trip.

Exit criteria:
- Canonical round-trip suites pass for return grammar corpus (generated + bootstrap where applicable).

#### RA-04 Return Gate Hardening
- Add dedicated gate slices:
  - `return_full_contract_gate`
  - `return_runtime_semantics_gate`
  - `return_ast_roundtrip_gate`
- Wire into `annotation_contract_gate` and aggregate SOTA required checks.

Exit criteria:
- Required CI checks enforce all RA gates.

## Workstream B: Semantic Annotation Closure (SA)

### SA Scope Inventory (from `grammars/semantic_annotation.ebnf`)
- Envelope and naming: `semantic_annotation`, `annotation`, `annotation_name`, `predefined_annotation`, `custom_annotation`
- Value model roots: `annotation_value`, `primitive_value`, `structured_value`, `expression_value`, `reference_value`
- Primitive values:
  - string family (`double_quoted_string`, `single_quoted_string`, `raw_string`, `multiline_string`, `template_string`)
  - numeric family (`integer_literal`, `decimal_literal`, `scientific_literal`, `hexadecimal_literal`, `binary_literal`, `octal_literal`)
  - booleans/null/identifiers
- Structured values:
  - arrays, objects (computed/shorthand/spread), tuples, sets, maps
- Expression values:
  - arithmetic, logical, comparison, conditional, function call, lambda
- References:
  - type references, rule/symbol/path/url references
- Specialized patterns:
  - precedence/constraint/performance/version/exception/platform families

### SA Milestones

#### SA-01 Parse Tree to Typed AST Closure
- Expand semantic typed AST beyond marker/raw forms to represent all grammar families.
- Implement generated-parser-backed parse tree to typed semantic AST conversion for conforming inputs.
- In non-bootstrap mode, eliminate raw fallback for grammar-conforming semantic annotations.

Exit criteria:
- 0 conforming semantic inputs land in generic raw fallback in non-bootstrap mode.
- 100% semantic grammar construct corpus maps to typed semantic AST.

#### SA-02 Typed Validation Closure
- Add stable diagnostics for every semantic construct family:
  - structural shape violations
  - value-domain violations
  - cross-field coherence violations
- Keep strictness policy controls, but ensure all conforming constructs validate cleanly.

Exit criteria:
- Complete diagnostic code catalog for semantic grammar families.
- Validator corpus covers positive and negative cases per family.

#### SA-03 Runtime Steering and Observability Closure
- For steering-capable constructs, guarantee parser/stimuli runtime behavior plus deterministic telemetry.
- For non-steering constructs, guarantee parse+validate+observable typed representation without silent drop.
- Keep layered policy model: hardcoded invariants minimal, policy via annotations first.

Exit criteria:
- Runtime behavior tests and telemetry assertions for all steering families.
- No dropped semantic constructs in generated parser/stimuli code paths.

#### SA-04 Semantic Gate Hardening
- Add dedicated gate slices:
  - `semantic_full_contract_gate`
  - `semantic_ast_roundtrip_gate`
  - `semantic_runtime_contract_gate`
- Wire into `annotation_contract_gate` and aggregate SOTA required checks.

Exit criteria:
- Required CI checks enforce all SA gates.

## Workstream C: Objective Proof Infrastructure (PX)

### PX-01 Construct Coverage Manifest
- Generate machine-readable rule/alternative manifests from:
  - `grammars/return_annotation.ebnf`
  - `grammars/semantic_annotation.ebnf`
- Track tested status per rule and per alternative.

### PX-02 Contract Corpus Expansion
- Build explicit corpora for:
  - parse acceptance/rejection
  - typed AST expected snapshots
  - runtime semantics expected outputs
  - canonical round-trip forms
- Include adversarial corpora:
  - deeply nested constructs,
  - mixed-feature compositions,
  - whitespace/comment stress,
  - boundary numeric/string/reference forms,
  - invalid near-miss forms to prove precise rejection.

### PX-02a EBNF-Based Stimuli Excellence (Binding)
- EBNF-driven stimuli generation is mandatory proof infrastructure, not optional convenience.
- Stimuli quality requirements:
  - construct-complete generation across all annotation grammar families,
  - controllable positive/negative/mutation mixes,
  - deterministic replay by seed,
  - gap-driven target injection until threshold closure,
  - shrinking/minimization for failing samples.
- Generated parser/stimuli behavior must be verified against expected intent outputs, not parse success alone.

### PX-03 Differential Closure Strategy
- Keep bootstrap suites for bootstrap guarantees.
- Keep comparable-only suites for generated-vs-bootstrap parity.
- Require zero mismatches for comparable corpora.

### PX-04 Annotation 100% Aggregate Gate
- Add single required aggregate:
  - `annotation_100_gate`
- Gate includes RA, SA, and PX checks and produces machine-readable pass report.
- Gate must include built-in coverage assertions derived from full grammar corpora.

Minimum sub-gates inside `annotation_100_gate`:
- `annotation_construct_coverage_gate`
  - proves 100% rule and alternative coverage for both annotation grammars.
- `annotation_typed_ast_gate`
  - proves all conforming samples map to typed AST without non-bootstrap fallback.
- `annotation_runtime_intent_gate`
  - proves generated parser/stimuli behavior matches expected intent outputs for construct corpus.
- `annotation_determinism_gate`
  - proves reproducible outputs across repeated runs (fixed seed and fixed inputs).
- `annotation_differential_parity_gate`
  - proves zero mismatches on comparable bootstrap/generated corpora.
- `annotation_stimuli_quality_gate`
  - proves stimuli generator quality thresholds (coverage closure, deterministic replay, gap-target convergence, and failure shrinking).
  - implemented baseline gate script:
    - `rust/scripts/annotation_stimuli_quality_gate.sh`
  - wired today into:
    - `make -C rust annotation_contract_gate`

Exit criteria:
- `annotation_100_gate` required in CI and aggregate SOTA policy.

## Ordered Execution Plan

1. RA-01, SA-01, PX-01
2. RA-02, SA-02, PX-02
3. RA-03, SA-03
4. RA-04, SA-04, PX-03
5. PX-04 and CI policy hard requirement

## Definition of Done (Roadmap End State)
Roadmap reaches 100% only when:
- `annotation_100_gate` is green in CI,
- comparable differential mismatches are zero,
- construct coverage manifests report 100% rule and alternative coverage for both annotation grammars,
- no conforming annotation input depends on bootstrap/raw fallback paths in non-bootstrap mode,
- runtime semantics tests pass for all steering-capable constructs.
- runtime semantics tests pass for all return-annotation constructs (not only steering semantics),
- CI branch protection requires `annotation_100_gate` and related annotation contract gates pre-merge.
- stimuli quality gate thresholds remain green in CI with no waived deficits.

## Change Log
- 2026-02-21: Advanced RA-01 from baseline to structural mapping by replacing span-based generated return conversion with rule-aware parse-tree mapping, aligning generated conversion semantics with bootstrap for extraction-index and zero/signed-zero positional handling, expanding generated conversion parity corpus tests, and broadening `return_runtime_semantics_gate` generated test coverage.
- 2026-02-20: Initial zero-compromise roadmap published for full return and semantic annotation closure with objective proof gates.
- 2026-02-20: Implemented `annotation_stimuli_quality_gate` baseline closed-loop verifier (return + semantic) with stage-level artifact and metric invariants; integrated into `annotation_contract_gate`.
- 2026-02-20: Advanced RA-02 runtime closure baseline by adding identifier literal + single-quoted string/object-key support to `UnifiedReturnAST`, wiring exhaustive transformer/validator/normalizer handling, and adding focused regression tests for these return construct families.
- 2026-02-20: Advanced RA-03 baseline by replacing generated return round-trip identity output in `test_runner` with shared typed canonical unparse output (`unparse_return_ast`) while preserving return parity gate zero-mismatch status.
- 2026-02-20: Advanced RA-04 baseline by adding dedicated return gate slices (`return_runtime_semantics_gate`, `return_ast_roundtrip_gate`, `return_full_contract_gate`) and wiring `return_full_contract_gate` into `annotation_contract_gate`.

## Appendix A: Exact Return Rule Inventory
Exact nonterminal inventory extracted from `grammars/return_annotation.ebnf`:

- `return_annotation`
- `arrow`
- `expression`
- `primary_expression`
- `extraction_expression`
- `extraction_target`
- `positive_integer`
- `spread_expression`
- `spreadable_expression`
- `spread_suffix`
- `property_access_expression`
- `array_access_expression`
- `accessor_base`
- `positional_reference`
- `string_literal`
- `string_content_double`
- `string_content_single`
- `number_literal`
- `float`
- `integer`
- `boolean_literal`
- `identifier`
- `object_literal`
- `object_properties`
- `object_property`
- `property_key`
- `array_literal`
- `array_elements`
- `array_element`
- `parenthesized`

## Appendix B: Exact Semantic Rule Inventory
Exact nonterminal inventory extracted from `grammars/semantic_annotation.ebnf`:

- `semantic_annotation`
- `annotation`
- `annotation_name`
- `predefined_annotation`
- `custom_annotation`
- `annotation_value`
- `primitive_value`
- `structured_value`
- `expression_value`
- `reference_value`
- `string_literal`
- `double_quoted_string`
- `single_quoted_string`
- `raw_string`
- `multiline_string`
- `template_string`
- `numeric_literal`
- `integer_literal`
- `decimal_literal`
- `scientific_literal`
- `hexadecimal_literal`
- `binary_literal`
- `octal_literal`
- `boolean_literal`
- `null_literal`
- `identifier_literal`
- `array_value`
- `array_element`
- `spread_element`
- `object_value`
- `object_property`
- `property_key`
- `computed_key`
- `computed_property`
- `shorthand_property`
- `spread_property`
- `tuple_value`
- `tuple_element`
- `named_tuple_element`
- `set_value`
- `set_element`
- `map_value`
- `map_entry`
- `arithmetic_expression`
- `multiplicative_expr`
- `power_expr`
- `unary_expr`
- `additive_op`
- `multiplicative_op`
- `unary_op`
- `primary_expr`
- `logical_expression`
- `logical_or_expr`
- `logical_and_expr`
- `logical_not_expr`
- `comparison_expression`
- `additive_comparison_expr`
- `comparison_op`
- `conditional_expression`
- `function_call`
- `function_name`
- `qualified_name`
- `function_argument`
- `positional_argument`
- `named_argument`
- `spread_argument`
- `lambda_expression`
- `lambda_parameter`
- `destructuring_parameter`
- `type_reference`
- `primitive_type`
- `generic_type`
- `union_type`
- `intersection_type`
- `function_type`
- `array_type`
- `optional_type`
- `rule_reference`
- `rule_reference_name`
- `symbol_reference`
- `path_reference`
- `absolute_path`
- `relative_path`
- `home_path`
- `url_reference`
- `precedence_value`
- `precedence_level`
- `precedence_associativity`
- `constraint_value`
- `constraint_type`
- `constraint_expression`
- `performance_value`
- `complexity_spec`
- `complexity_expr`
- `memory_spec`
- `memory_amount`
- `memory_unit`
- `timing_spec`
- `time_amount`
- `time_unit`
- `version_value`
- `semantic_version`
- `version_range`
- `exception_spec`
- `exception_type`
- `platform_spec`
- `platform_name`
- `whitespace`
- `line_comment`
- `block_comment`
- `doc_comment`
