# PGEN SOTA Implementation Roadmap (Living)

Last updated: 2026-02-20

## Mission
Build PGEN into a state-of-the-art parser and stimuli generation platform with production-grade return/semantic annotation support, suitable for embedding in high-rigor systems (SystemVerilog/VHDL tooling, regex engines, and similar domains).

## Status Legend
- `Not Started`
- `In Progress`
- `Blocked`
- `Done`

## Pillar Tracker

| Pillar | Current Status | Target Outcome |
|---|---|---|
| 1. Bootstrap Trust and Fixed-Point Reproducibility | In Progress | Repeated bootstrap cycles are byte-identical for annotation grammars. |
| 2. Normative Annotation Specification | In Progress | One normative spec for return/semantic annotation syntax + semantics. |
| 3. Typed Annotation Validation | In Progress | Compile-time validation of references/transforms with precise diagnostics. |
| 4. Bootstrap vs Generated Behavioral Contract | In Progress | Explicitly tracked differences with required tests and closure plan. |
| 5. Industrial Frontend Support (SV/VHDL Readiness) | Not Started | Preprocess/lex/parse pipeline robust for real-world HDL sources. |
| 6. Ambiguity Handling and Recovery | In Progress | Deterministic branch resolution and production-grade error recovery. |
| 7. Coverage-Guided Semantic Stimuli | In Progress | Feedback loop that drives branch/rule/annotation coverage upward. |
| 8. Differential Validation vs External Parsers | In Progress | Continuous mismatch detection against trusted external tools. |
| 9. Performance and Scalability SLAs | In Progress | Enforced throughput/memory/latency budgets in CI. |
| 10. Embedding-Grade APIs and Contracts | In Progress | Stable crate API, deterministic behavior, and versioned contracts. |
| 11. Security and Robustness Hardening | In Progress | Fuzzed, bounded, and resilient parser/stimuli runtime. |
| 12. SOTA Exit Criteria Gate | In Progress | CI-enforced release gates with objective pass thresholds. |

## Execution Plan (Ordered)

### Phase A (Now)
- [x] Create living roadmap document and track implementation status here.
- [x] Add fixed-point bootstrap gate script for return/semantic parser artifacts.
- [x] Add `make fixed_point_gate` target for local and CI usage.
- [x] Wire `fixed_point_gate` into CI as required pre-merge gate.
- [x] Increase gate strictness from 2-cycle to 3-cycle minimum in CI.

### Phase B (Next)
- [x] Implement typed return annotation validator (`$n`, extraction, spread, object/array shape checks).
- [x] Implement typed semantic annotation validator (transform function signatures and argument checks).
- [x] Emit structured diagnostics with stable error codes.
- [x] Extend validator coverage to include rule-aware capture bounds and branch-shape compatibility.
- [x] Add strict-mode failure policy into standard CI gates (not only env-triggered mode).

### Phase C
- [x] Add coverage-guided semantic fuzzing loop with seed replay and corpus minimization.
- [x] Add shrinking for failing stimuli and parseability counterexamples.
- [x] Add gap-driven generator priorities (rule/branch/annotation coverage targets).

### Phase D
- [x] Add differential harness against external parser/tool baselines.
- [x] Add performance benchmark suite and CI thresholds.
- [x] Finalize embedding API stability and versioning policy.

### Phase E
- [x] Add differential mismatch taxonomy + baseline regression gate (`new mismatch only`) with tracked baseline snapshots.
- [x] Wire `differential_regression_gate` into CI as required pre-merge check with report artifact retention.
- [x] Author and maintain comprehensive PGEN User Guide (EBNF, return/semantic annotations, coverage flows, stimuli generation, automation workflows, troubleshooting).

### Phase F (Current)
- [x] Publish living normative annotation specification with explicit bootstrap/full/validator contracts.
- [x] Add executable bootstrap contract suites for inferred built-in return/semantic parser behaviors.
- [x] Add `make annotation_contract_gate` to enforce validator + built-in contract suites.
- [x] Wire `annotation_contract_gate` into CI as required pre-merge check.
- [x] Add shared bootstrap/generated annotation contract suites and enforce them in gate paths.
- [x] Add semantic leverage contract gate (`semantic_usage_gate`) for parser/stimuli steering behavior.
- [x] Align validator, parser codegen, and stimuli hinting on a shared canonical semantic transform parser.
- [x] Add annotation robustness gate (advanced suites + generated parseability/coverage/gap checks) and enforce it via `annotation_contract_gate`.

### Phase G (Current)
- [x] Add embedding API input bounds (`ParseLimits`) with stable diagnostics for oversized/invalid inputs.
- [x] Extend embedding API contract docs with limit behavior and new diagnostic codes.

### Phase H (New): Rust-Native EBNF Frontend Migration
- [x] Add executable EBNF frontend readiness report/gate for `grammars/ebnf.ebnf`, `grammars/json.ebnf`, and `grammars/regex.ebnf` (Perl `EBNF -> JSON`, Rust `JSON -> parser`, Rust stimuli generation).
- [x] Fix `grammars/ebnf.ebnf` compatibility gaps so readiness strict mode is green for all tracked grammars.
- [x] Add dual-run differential harness between Perl `ebnf_to_json.pl` and Rust-native EBNF parser (`generated/ebnf.rs`) once Rust EBNF parser generation path is available.

### Phase I (New): SOTA Exit Criteria Aggregation
- [x] Add aggregate `make sota_exit_gate` to execute required release-grade checks in one command.
- [x] Add script-backed summary/log artifacts for aggregate gate runs (`rust/target/sota_exit_gate`).
- [x] Add CI workflow `sota-exit-gate` to run the aggregate gate and retain artifacts.
- [x] Promote EBNF frontend strict mode (`ebnf_frontend_gate`) to required inside aggregate gate once `grammars/ebnf.ebnf` compatibility is fixed.
- [x] Define and enforce explicit release pass policy for aggregate gate output (for example branch protection + release checklist criteria).
- [x] Add non-bootstrap annotation end-to-end gate (`annotation_nonbootstrap_e2e_gate`) and enforce it in both standalone CI and aggregate SOTA required-check policy.

### Phase J (New): Semantic Steering Control Surface + Return Completeness
- [x] Publish semantic steering control matrix with parser/stimuli control taxonomy, current support status, and target tiers (`PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`).
- [x] Capture layered control architecture decision (minimal built-in invariants + annotation-driven policy surface + hard precedence rules).
- [x] `P0` Implement typed semantic directive registry (name-based routing) and promote from parse-only to steering for selected directives.
- [x] `P0` Implement unknown-directive handling policy modes (`warn` and `strict`) with stable diagnostics.
- [x] `P0` Implement parser/stimuli precedence+associativity steering controls.
- [x] `P0` Implement parser/stimuli value-domain constraint controls (`range/enum/regex/len` style directives) baseline.
- [x] `P1` Implement semantic directive conflict-resolution baseline (deterministic `priority > precedence` policy and duplicate directive override diagnostics).
- [x] `P1` Expand semantic conflict diagnostics to unsatisfiable multi-directive intersections.
- [x] `P1` Add comparable-corpus return parity gate (`make return_parity_gate`) and enforce inside `annotation_contract_gate`.
- [x] `P1` Drive return-annotation differential mismatches to zero and enforce stricter return parity closure criteria (tracked debt now 0).

### Phase K (New): Ambiguity Handling and Recovery Kickoff
- [x] Add grammar-aware ambiguity prefix diagnostics (`W_GRAM_AMBIGUOUS_PREFIX`) for top-level alternation branches sharing identical leading quoted terminals.
- [x] Extend ambiguity diagnostics from literal-prefix heuristics to nullable/first-set overlap analysis.
- [x] Add branch-policy + recovery hint contract surface (`@branch_policy`, `@recover`, `@sync`, `@panic_until`) with validator diagnostics and parser/stimuli integration plan.
- [x] Promote recovery-hint contracts to executable parser runtime baseline (`@recover` + `@sync/@panic_until` token-scan recovery hooks with deterministic fallback behavior).
- [x] Add stimuli-side SC-07 recovery baseline: OR-failure fallback marker emission driven by typed `@recover/@sync/@panic_until`.
- [x] Expand SC-07 stimuli generation with dedicated recovery-focused modes (`recovery_biased`, `near_sync_negative`) for near-sync negative-case synthesis.
- [x] Add structured parser recovery event reporting baseline (`RecoveryEvent`/`RecoveryMarkerKind` + event accessors on generated parsers).
- [x] Add typed rule-local recovery budget control (`@recover_budget`) with validator diagnostics + parser runtime enforcement.
- [x] Expand SC-07 with scoped recovery budget controls (`@recover_parse_budget`, `@recover_global_budget`) plus validator diagnostics and runtime counter accessors.
- [x] Start SC-09 cross-field/cross-capture constraint contract baseline with typed validator payload/coherence diagnostics (`@constraint/@requires/@implies`).
- [x] Promote SC-09 to parser runtime baseline by enforcing `@constraint/@requires/@implies` contracts in generated rule methods (reference resolution + relational expression checks + implication checks).
- [x] Promote SC-09 to stimuli runtime baseline by enforcing relational constraints during sequence synthesis with retry-based constraint satisfaction.
- [x] Harden SC-09 stimuli nested reference synthesis by supporting dotted named/positional path resolution for structured capture values (for example `lhs.id`, `$1.id.len`).
- [x] Harden SC-09 stimuli retry exhaustion diagnostics with ranked unsatisfiable-contract reporting (`relational_failures`, `generation_failures`, `top_violations`, `likely_unsatisfiable`).
- [x] Harden SC-09 stimuli nested reference synthesis for non-structured object-like captures (`=/:` pairs, wrapper-aware parsing, dotted-key path materialization).
- [x] Start SC-10 typed semantic coverage-target hinting (`@coverage_target/@critical_path`) with validator contracts and stimuli coverage/gap steering integration.
- [x] Extend SC-10 to parser runtime instrumentation hooks (`CoverageTargetEvent`, selected-branch tagging, rule/branch hit counters + accessors) while keeping built-in behavior minimal/invariant-only.
- [x] Promote selected semantic warning diagnostics to strict-mode errors under explicit policy controls (`PGEN_STRICT_SEMANTIC_WARNING_CODES`) with bootstrap-compatible defaults.

## Current Sprint: Pillar 1

### Completed in this sprint
- Added a reproducibility gate that regenerates return/semantic annotation JSON and parser outputs for multiple cycles and asserts byte-identical outputs between cycle 1 and subsequent cycles.

### Remaining for Pillar 1 completion
- Branch protection rule should require the `fixed-point-gate` check before merge.

## Risks and Mitigations
- Risk: Non-deterministic codegen details (ordering, paths, timestamps) can create false drifts.
  - Mitigation: Fixed output paths per cycle, byte-level comparisons, and explicit diff output on mismatch.
- Risk: Gate bypass in local workflows.
  - Mitigation: Add Make target now; enforce in CI next.
- Risk: Bootstrap/generated behavior drifts without visibility.
  - Mitigation: Maintain conformance tests and feature matrix tracking as required checklists.

## Change Log (Roadmap Updates)
- 2026-02-18: Initialized roadmap and marked Pillar 1 implementation started.
- 2026-02-18: Added GitHub Actions `fixed-point-gate` workflow and started Phase B validator implementation with structured diagnostics.
- 2026-02-18: Extended annotation validator with grammar-aware branch/capture checks and integrated grammar-aware validation into parser generation.
- 2026-02-18: Enforced strict annotation validation policy in standard CI gate path and switched CI fixed-point runs to 3 cycles by default.
- 2026-02-18: Added fixed-point drift artifact upload/retention policy in CI (failure-only upload of `rust/target/fixed_point_gate` snapshots and diffs).
- 2026-02-18: Added coverage-guided fuzz loop mode with deterministic per-seed replay and greedy corpus minimization for stimuli generation.
- 2026-02-18: Added shrinking for parseability counterexamples and failing stimuli traces (delta-debug-style minimization in replay and parseability-failure diagnostics).
- 2026-02-18: Added gap-priority generation mode that applies reachable targets from a gap report as branch/rule bias in count-based stimuli generation.
- 2026-02-18: Started Phase D by adding generated-vs-bootstrap differential harness mode in `test_runner` with JSON mismatch reports and Makefile automation (`make differential_report`).
- 2026-02-18: Completed Phase D performance gate by adding `perf_bench`, `make performance_gate`, threshold policy (`rust/perf/thresholds.json`), and CI workflow wiring (`performance-gate`).
- 2026-02-18: Completed Phase D embedding API stabilization with versioned `pgen::embedding_api` contracts, deterministic structured parse outcomes, and `make embedding_api_gate`.
- 2026-02-18: Started Phase E by adding mismatch taxonomy + baseline closure tracking in differential mode and a regression-only gate (`make differential_regression_gate`) backed by tracked baseline snapshots under `rust/test_data/differential_baseline/`.
- 2026-02-18: Added CI workflow `differential-regression-gate` to enforce `make differential_regression_gate` on PR/main and retain differential report artifacts.
- 2026-02-18: Published initial comprehensive end-user guide in `PGEN_USER_GUIDE.md` and linked it from `README.md` (living document for onboarding + full feature usage).
- 2026-02-18: Started Pillar 2 by publishing `PGEN_ANNOTATION_NORMATIVE_SPEC.md`, adding bootstrap contract suites (`builtin_contract.json`), and wiring `make annotation_contract_gate`.
- 2026-02-18: Added CI workflow `annotation-contract-gate` to enforce `make annotation_contract_gate` on PR/main.
- 2026-02-18: Added shared bootstrap/generated contract suites (`normative_shared_contract.json`) and extended `annotation_contract_gate` with `annotation_shared_contract_gate`.
- 2026-02-18: Added semantic leverage contract coverage (`semantic_usage_*` tests) and wired `semantic_usage_gate` into `annotation_contract_gate`.
- 2026-02-18: Added shared canonical semantic transform parsing (`semantic_transform.rs`) and wired validator/parser/stimuli to it, including path-type and noncanonical fallback tests.
- 2026-02-18: Started Pillar 11 by hardening embedding API parsing with bounded input limits (`ParseLimits`, `E_INPUT_TOO_LARGE`, `E_INVALID_LIMITS`) and updated contract docs.
- 2026-02-19: Started Phase H by adding `make ebnf_frontend_readiness` / `make ebnf_frontend_gate` and script-backed reporting of current frontend status across `ebnf/json/regex` grammar flows.
- 2026-02-19: Hardened Phase F with `make annotation_robustness_gate` (advanced bootstrap/generated annotation suites plus generated parseability+coverage/gap checks) and enforced it inside `annotation_contract_gate`.
- 2026-02-19: Started Pillar 12/Phase I by adding aggregate `make sota_exit_gate`, script-backed run summaries under `rust/target/sota_exit_gate`, CI workflow `sota-exit-gate`, and refreshed tracked differential baselines so aggregate required checks run with explicit known-drift accounting.
- 2026-02-19: Completed explicit aggregate release policy enforcement by adding tracked policy config (`rust/config/sota_exit_policy.env`), policy-driven required-check execution in `sota_exit_gate`, and release policy checklist doc (`PGEN_RELEASE_POLICY.md`).
- 2026-02-19: Started Phase J by publishing semantic steering control matrix (`PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`) and codifying the no-compromise return-annotation completeness contract.
- 2026-02-19: Refined Phase J with explicit layered built-in-vs-annotation control decision and P0/P1 implementation priorities for semantic steering promotion.
- 2026-02-19: Completed Phase J P0 directive-registry foundation: added typed semantic directive registry + unknown-directive warn/strict policy with stable diagnostics, and wired directive-aware steering routing in parser codegen and stimuli generation.
- 2026-02-19: Completed Phase J P0 precedence/associativity steering baseline by adding semantic `priority/precedence` branch tie-break controls and `associativity` tie policy routing in parser codegen and stimuli branch selection.
- 2026-02-19: Completed Phase J P0 value-domain steering baseline by wiring semantic `enum/range/len/regex` constraints into parser value guards and stimuli generation candidate selection, and added typed validator payload diagnostics plus semantic usage gate coverage for value-domain directives.
- 2026-02-19: Completed Phase J deterministic conflict-resolution baseline by enforcing `priority > precedence` policy in parser/stimuli steering and adding validator diagnostics for `priority+precedence` overlap and duplicate known-directive last-wins resolution.
- 2026-02-19: Completed Phase J P1 unsatisfiable value-domain intersection diagnostics by adding validator detection for contradictory `@enum` + (`@len`/`@range`/`@regex`) combinations with stable warning code `W_SEM_UNSATISFIABLE_VALUE_DOMAIN`.
- 2026-02-19: Added comparable-only differential mode and wired `make return_parity_gate` into `annotation_contract_gate` to enforce zero mismatches on expectation-aligned return parity corpus while preserving tracked bootstrap-only drift separately.
- 2026-02-19: Burned down return differential debt from 9 to 7 by enabling empty-arrow (`->`) generated parsing parity and enforcing positive extraction index (`::0` rejection) in return grammar + regenerated artifacts + baseline refresh.
- 2026-02-19: Burned down return differential debt from 7 to 2 by tightening bootstrap return parser behavior (leading-arrow whitespace normalization, strict trailing-modifier rejection, strict comma-list segment rejection), updating builtin return contract corpus/grammar inference, and refreshing baseline snapshots.
- 2026-02-19: Completed Phase J P1 return differential closure by extending bootstrap positional/accessor-chain parsing (signed positional refs, chained postfix including extraction/property/index and parenthesized index expressions), reducing tracked return mismatch debt from 2 to 0, and refreshing return differential baseline + parity/regression gate runs.
- 2026-02-19: Completed Phase H `ebnf.ebnf` frontend compatibility by fixing include directive capture in `fx/specs/ebnf.spec` (consume full include call, parse args in action code), restoring `ebnf_to_json.pl` conversion success for `grammars/ebnf.ebnf`, and turning strict EBNF frontend enforcement on in aggregate SOTA policy (`PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`).
- 2026-02-19: Added standalone CI workflow `annotation-nonbootstrap-e2e-gate` and promoted `annotation_nonbootstrap_e2e_gate` into aggregate SOTA required checks (`rust/scripts/sota_exit_gate.sh` + `rust/config/sota_exit_policy.env`).
- 2026-02-19: Completed Phase H dual-run differential operationalization by adding `make ebnf_frontend_dual_run_diff`/`make ebnf_frontend_dual_run_gate`, script + Rust report binary (`rust/scripts/ebnf_frontend_dual_run_diff_gate.sh`, `rust/src/bin/ebnf_dual_run_diff.rs`), standalone CI workflow (`ebnf-frontend-dual-run-diff`), and aggregate SOTA policy/workflow wiring with optional strictness controls.
- 2026-02-20: Started Pillar 6/Phase K by adding grammar-aware ambiguity diagnostics (`W_GRAM_AMBIGUOUS_PREFIX`) in annotation validator grammar-aware pass with unit tests for overlapping-vs-distinct literal-prefix alternation branches.
- 2026-02-20: Completed Phase K nullable/FIRST overlap milestone by adding `W_GRAM_FIRST_SET_OVERLAP` and `W_GRAM_NULLABLE_BRANCH_SHADOW` diagnostics (including nullable-prefix rule-reference overlap coverage) in the grammar-aware annotation validator pass.
- 2026-02-20: Completed Phase K branch-policy/recovery contract surface by adding typed directive parsing/validation (`@branch_policy`, `@recover`, `@sync`, `@panic_until`), parser+stimuli branch-policy steering baseline (`longest_match`/`ordered`/`priority_first`), and staged recovery-hint integration signaling.
- 2026-02-20: Completed Phase K recovery runtime baseline by wiring generated parser OR-failure recovery hooks (`recover_with_hints`) that consume typed `@recover/@sync/@panic_until` directives, scan for nearest panic/sync tokens with deterministic precedence, advance parser position with EOF fallback, and continue parsing with explicit recovery logging.
- 2026-02-20: Added Phase K stimuli-side recovery baseline by wiring OR-generation fallback marker emission in `StimuliGenerator` for effective `@recover` rules (first non-empty `@panic_until`, then `@sync`) and covering with semantic usage regression tests.
- 2026-02-20: Added Phase K structured recovery reporting baseline in generated parsers by introducing typed recovery events (`RecoveryEvent`, `RecoveryMarkerKind`), parser accessors (`recovery_events`, `take_recovery_events`, `recovery_event_count`), and event recording for token-based and EOF fallback recovery paths.
- 2026-02-20: Added Phase K typed rule-local recovery budget control (`@recover_budget`) by extending directive registry + validator diagnostics (`W_SEM_INVALID_RECOVER_BUDGET_PAYLOAD`, `W_SEM_RECOVER_BUDGET_WITHOUT_RECOVER`) and wiring generated parser enforcement (budget exhaustion blocks additional recoveries for the same rule in a parse run).
- 2026-02-20: Expanded Phase K SC-07 scoped recovery controls by adding typed parse/global budget directives (`@recover_parse_budget`, `@recover_global_budget`), new validator diagnostics (`W_SEM_INVALID_RECOVER_PARSE_BUDGET_PAYLOAD`, `W_SEM_INVALID_RECOVER_GLOBAL_BUDGET_PAYLOAD`, `W_SEM_RECOVER_PARSE_BUDGET_WITHOUT_RECOVER`, `W_SEM_RECOVER_GLOBAL_BUDGET_WITHOUT_RECOVER`), runtime enforcement in generated parser recovery hooks, and parser-level counters (`recovery_parse_count`, `recovery_global_count`).
- 2026-02-20: Expanded Phase K SC-07 stimuli steering beyond OR-failure fallback by adding dedicated recovery-focused generation modes (`recovery_biased`, `near_sync_negative`) in `StimuliGenerator` and CLI wiring (`--recovery-stimuli-mode`) with semantic usage regression coverage.
- 2026-02-20: Started SC-09 contract baseline by adding typed validator parsing for `@constraint/@requires/@implies`, stable payload diagnostics (`W_SEM_INVALID_CONSTRAINT_PAYLOAD`, `W_SEM_INVALID_REQUIRES_PAYLOAD`, `W_SEM_INVALID_IMPLIES_PAYLOAD`), and relational coherence warning (`W_SEM_RELATIONAL_HINT_WITHOUT_CONSTRAINT`).
- 2026-02-20: Promoted SC-09 to parser runtime baseline by wiring generated relational enforcement (`@requires` reference checks, `@constraint` expression evaluation, `@implies` antecedent/consequent gating) with contextual parse errors and semantic usage coverage.
- 2026-02-20: Promoted SC-09 to stimuli runtime baseline by adding relational constraint-aware sequence retries (`@requires/@constraint/@implies`) with semantic usage coverage for cross-capture filtering and implication steering.
- 2026-02-20: Hardened SC-09 stimuli nested-path resolution by adding structured capture traversal for named/positional dotted references (`lhs.id`, `$1.id`, `$3.id.len`) in relational checks and added semantic usage regression coverage for nested named and positional path contracts.
- 2026-02-20: Hardened SC-09 retry-exhaustion diagnostics in stimuli synthesis by adding aggregated relational failure reporting (`relational_failures`, `generation_failures`, ranked `top_violations`, and `likely_unsatisfiable` flag) plus semantic usage regression coverage for unsatisfiable contracts.
- 2026-02-20: Hardened SC-09 stimuli nested-path resolution for non-structured object-like captures by adding wrapper-aware loose key/value parsing (`=/:`, `,`/`;`/newline delimiters), dotted-key materialization (for example `meta.id=AA`), and semantic usage regression coverage for named/positional non-structured relational references.
- 2026-02-20: Started SC-10 semantic coverage-target steering baseline by adding typed parser helpers and validator contracts for `@coverage_target/@critical_path` (including coherence warning `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`), and wiring stimuli branch guidance + gap-report/target priority bonuses from semantic coverage hints.
- 2026-02-20: Extended SC-10 to parser runtime instrumentation baseline by wiring generated-parser coverage-target hooks (`record_coverage_target_event`), selected-branch tagging for OR rules, typed event surface (`CoverageTargetEvent`), and parser accessors/counters (`coverage_target_events`, `take_coverage_target_events`, `coverage_target_rule_hits`, `coverage_target_branch_hits`).
- 2026-02-20: Added strict semantic warning promotion policy controls in validator/codegen (`PGEN_STRICT_SEMANTIC_WARNING_CODES`) with targeted strict-default escalations for malformed SC-10 payload diagnostics (`W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`, `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`), plus wildcard/all/none policy overrides and regression tests.
