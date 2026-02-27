# PGEN SOTA Implementation Roadmap (Living)

Last updated: 2026-02-27

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
| 5. Industrial Frontend Support (SV/VHDL Readiness) | In Progress | Preprocess/lex/parse pipeline robust for real-world HDL sources. |
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
- [x] Start SC-04 token-class steering baseline with typed validator contracts and parser/stimuli precedence policy (`@pattern > @charset > @token_class`).
- [x] Start SC-09 cross-field/cross-capture constraint contract baseline with typed validator payload/coherence diagnostics (`@constraint/@requires/@implies`).
- [x] Promote SC-09 to parser runtime baseline by enforcing `@constraint/@requires/@implies` contracts in generated rule methods (reference resolution + relational expression checks + implication checks).
- [x] Promote SC-09 to stimuli runtime baseline by enforcing relational constraints during sequence synthesis with retry-based constraint satisfaction.
- [x] Harden SC-09 stimuli nested reference synthesis by supporting dotted named/positional path resolution for structured capture values (for example `lhs.id`, `$1.id.len`).
- [x] Harden SC-09 stimuli retry exhaustion diagnostics with ranked unsatisfiable-contract reporting (`relational_failures`, `generation_failures`, `top_violations`, `likely_unsatisfiable`).
- [x] Harden SC-09 stimuli nested reference synthesis for non-structured object-like captures (`=/:` pairs, wrapper-aware parsing, dotted-key path materialization).
- [x] Start SC-10 typed semantic coverage-target hinting (`@coverage_target/@critical_path`) with validator contracts and stimuli coverage/gap steering integration.
- [x] Extend SC-10 to parser runtime instrumentation hooks (`CoverageTargetEvent`, selected-branch tagging, rule/branch hit counters + accessors) while keeping built-in behavior minimal/invariant-only.
- [x] Promote selected semantic warning diagnostics to strict-mode errors under explicit policy controls (`PGEN_STRICT_SEMANTIC_WARNING_CODES`) with bootstrap-compatible defaults.
- [x] Start SC-11 negative-case semantic steering (`@invalid_case/@negative`) with typed validator contracts and parser/stimuli runtime baseline behavior.
- [x] Start SC-12 determinism partitioning hints (`@seed_group/@deterministic_group`) with typed validator contracts and deterministic stimuli seed-partition routing baseline.
- [x] Promote SC-12 to parser-side deterministic partition steering baseline (ordered OR branch partition offsets + typed parser partition events/counters).
- [x] Harden SC-12 embedder controls with generated-parser runtime partition mode overrides (`AnnotationDriven`/`ForceEnabled`/`ForceDisabled`) and runtime-effective ordered-branch partition resolution.
- [x] Promote SC-06 to Tier-4 contract gate by adding dedicated SC-06 semantic contract slices + branch-policy/weighting runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-07 to Tier-4 contract gate by adding dedicated SC-07 semantic contract slices + recovery/sync runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-09 to Tier-4 contract gate by adding dedicated SC-09 semantic contract slices + relational runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-10 to Tier-4 contract gate by adding dedicated SC-10 semantic contract slices + coverage-target runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-11 to Tier-4 contract gate by adding dedicated SC-11 semantic contract slices + negative-case runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-12 to Tier-4 contract gate by adding dedicated SC-12 semantic contract slices + deterministic partition runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-04 to Tier-4 contract gate by adding dedicated SC-04 semantic contract slices + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-05 to Tier-4 contract gate by adding dedicated SC-05 semantic contract slices + precedence/associativity runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Promote SC-08 to Tier-4 contract gate by adding dedicated SC-08 semantic contract slices + value-domain runtime contracts + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).
- [x] Harden SC-03 to Tier-4 by adding dedicated directive-routing/strict-policy contract slices + differential mismatch taxonomy parity checks inside `annotation_contract_gate` (CI-enforced).

### Phase L (New): Annotation 100% Closure (Return + Semantic)
- [x] Publish dedicated zero-compromise closure roadmap mapped to full annotation grammars:
  - `PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`
- [x] Implement annotation proof doctrine gates with uncompromising quality criteria:
  - full construct/alternative coverage manifests,
  - typed-AST no-fallback assertions in non-bootstrap mode,
  - runtime-intent conformance suites for parser/stimuli behavior,
  - determinism and comparable differential parity enforcement.
- [x] Add `annotation_stimuli_quality_gate` as required pre-merge proof for EBNF-based stimuli excellence:
  - construct-complete annotation stimuli coverage,
  - deterministic replay guarantees,
  - gap-target convergence thresholds,
  - failure-shrinking/minimization coverage.
- [x] Implement generated-parser-backed typed AST closure for full return grammar in non-bootstrap path (remove bootstrap typed-AST fallback for conforming inputs).
  - Progress (2026-02-21): non-bootstrap return entry parsing now requires generated parser success and no longer falls back to bootstrap entry parsing.
  - Progress (2026-02-22): added full generated-pass return corpus typed-AST closure proof (`generated_return_tree_to_typed_ast_matches_bootstrap_for_expected_pass_return_corpus`) with expectation-aware suite discovery and bootstrap parity assertions for comparable cases.
- [x] Implement generated-parser-backed typed AST closure for full semantic grammar in non-bootstrap path (remove raw/marker fallback for conforming inputs).
  - Progress (2026-02-21): generated semantic round-trip wrapper now uses generated parse-tree conversion (`UnifiedSemanticAST::parse_generated_semantic_annotation`) instead of identity output, and regression coverage was added for transform and named-raw directive conversion.
  - Progress (2026-02-22): non-bootstrap pipeline now routes named semantic annotations through generated parse-tree conversion (`parse_generated_semantic_annotation_entry`) and removes bootstrap marker-based legacy fallback for non-directive string payloads (legacy payloads remain raw unless bootstrap mode is explicitly enabled); generated round-trip canonicalization now reconstructs `@name: value` from parsed entry name + payload AST.
  - Progress (2026-02-22): semantic differential regression/parity gates are now expectation-aligned (`--differential-comparable-only`) with a zero-mismatch semantic comparable baseline, so bootstrap-only/legacy semantic corpus cases no longer count as open parity debt.
  - Progress (2026-02-22): added full generated-pass semantic corpus typed conversion contract test (`generated_semantic_tree_to_ast_matches_expected_pass_semantic_corpus_contract`) covering entry/direct conversion parity, transform-vs-raw directive mapping invariants, canonical `@name: value` reconstruction stability, and bootstrap-parity checks for non-transform comparable payloads.
  - Progress (2026-02-22): hardened non-bootstrap named semantic extraction to require generated parse-tree conversion when backend parseability validates, while preserving raw named payloads when backend rejects (compatibility with currently unsupported transform payload forms); validated via `annotation_nonbootstrap_e2e_gate`.
  - Progress (2026-02-22): closure validation completed via aggregate typed-AST proof gate (`make annotation_typed_ast_gate`) covering return/semantic runtime semantics and non-bootstrap end-to-end annotation flow.
- [x] Add full-contract gates (`return_full_contract_gate`, `semantic_full_contract_gate`, `annotation_100_gate`) and make them required in CI/SOTA aggregate policy.

### Phase M (New): Cross-EBNF Closed-Loop Quality (Non-Annotation Loop)
- [x] Split quality enforcement into two independent loops:
  - annotation-specialized loop (`annotation_stimuli_quality_gate`)
  - non-annotation loop (`ebnf_stimuli_quality_gate`)
- [x] Add contract-driven non-annotation grammar roster:
  - `rust/test_data/grammar_quality/ebnf_stimuli_contract.json`
- [x] Enforce strict non-annotation EBNF loop invariants per grammar:
  - frontend `EBNF -> JSON` success,
  - parser generation success,
  - deterministic 4-stage stimuli/coverage/gap closed-loop checks,
  - no-regression/target-summary/final-gap integrity assertions.
- [x] Promote `ebnf_stimuli_quality_gate` into aggregate SOTA required-check policy.
- [x] Promote parseability validation from optional to required grammar-by-grammar as parser registry coverage expands beyond annotation grammars.
  - Progress (2026-02-21): `builtin_return_annotation` is parser-registry-backed for generated parseability checks and promoted to `require_parseability=true`.
  - Closure (2026-02-22): `builtin_semantic_annotation` now uses a matching builtin-semantic parseability adapter in `parser_registry` (bootstrap semantic contract behavior via `UnifiedSemanticAST::parse_bootstrap`), and is promoted to `require_parseability=true` in `rust/test_data/grammar_quality/ebnf_stimuli_contract.json`.
  - Progress (2026-02-26): added parser-registry `ebnf` parseability adapter (feature-gated by `ebnf_dual_run`), promoted `ebnf` contract entry to `require_parseability=true`, and hardened `ebnf_stimuli_quality_gate` with two-phase build/bootstrap (`generated_parsers` then `generated_parsers+ebnf_dual_run` after generating `generated/ebnf.rs`) so required parseability is executable in gate paths.

### Phase N (New): Generated Stimuli Module Artifacts (`<grammar>_stimuli.rs`)
- [x] Add explicit AST-pipeline generation mode for Rust stimuli modules (for example `--generate-stimuli-module`) from both JSON and EBNF frontend inputs.
- [x] Define deterministic artifact contract:
  - output path pattern: `generated/<grammar>_stimuli.rs`,
  - stable exported API surface for embedding,
  - deterministic output under fixed grammar + config.
- [x] Preserve current in-memory stimuli path as default behavior while enabling file artifact generation via explicit opt-in CLI flag.
- [x] Add parity gate between in-memory stimuli generation and generated stimuli-module behavior:
  - same grammar,
  - same seed/config,
  - equivalent acceptance/coverage/gap outcomes.
- [x] Add gate target (for example `stimuli_module_parity_gate`) and wire into aggregate SOTA required-check policy once parity is stable.
- [x] Extend User Guide and normative docs with:
  - when to use in-memory vs generated stimuli module,
  - embedding workflow examples for `generated/<grammar>_stimuli.rs`,
  - deterministic replay/seed compatibility guarantees.

### Phase O (New): Industrial Frontend Readiness Kickoff (SV/VHDL)
- [x] Add executable HDL frontend readiness report target (`make hdl_frontend_readiness`) with state artifacts under `rust/target/hdl_frontend_gate`.
- [x] Add strict HDL frontend gate target (`make hdl_frontend_gate`) that fails on missing/failing grammar flows.
- [x] Define tracked initial grammar roster for readiness (`systemverilog`, `vhdl`) and report missing grammar files explicitly as `not_ready`.
- [x] Add first executable SystemVerilog seed grammar (`grammars/systemverilog.ebnf`) from IEEE 1800 markdown syntax sections and drive `systemverilog` HDL readiness row to pass.
- [ ] Add first executable VHDL seed grammar (`grammars/vhdl.ebnf`) and turn strict HDL frontend readiness green for both tracked HDL grammars.
- [ ] Decide aggregate SOTA policy integration mode for HDL readiness (informational first, then required strict once seed grammars stabilize).

### Phase P (New): SOTA SystemVerilog Parser + Stimuli Semantic Closure (Nexsim)
Objective: deliver a production-grade SystemVerilog parser/stimuli flow for Nexsim where acceptance requires both syntax correctness and semantic correctness.
Execution contract: `Phase Q` (preprocessor closure) is a hard prerequisite for strict Phase P closure.

Toolbox baseline to leverage end-to-end:
- `grammars/systemverilog.ebnf`
- `grammars/ebnf.ebnf`
- `grammars/return_annotation.ebnf` -> `generated/return_annotation_parser.rs`
- `grammars/semantic_annotation.ebnf` -> `generated/semantic_annotation_parser.rs`

- [ ] Freeze `systemverilog_core_v0` contract corpus and add `sv_stimuli_quality_gate`:
  - `EBNF -> JSON -> parser -> stimuli -> parse_full -> coverage/gap -> replay`.
- [ ] Add `SV_GRAMMAR_COVERAGE_MATRIX.md` mapped to IEEE syntax anchors (Annex-A-aligned sections) and track per-rule implementation status.
- [ ] Build syntax-closure burn-down loop:
  - grow `systemverilog.ebnf` clause-by-clause under deterministic no-regression gates.
- [ ] Build semantic-closure profile and validator pass for generated SV stimuli:
  - declaration-before-use,
  - scope/package import resolution,
  - port binding legality,
  - type/width compatibility,
  - context legality (`always_ff`, `always_comb`, generate constraints).
- [ ] Add SV stimuli generation modes with semantic steering:
  - `sv_snippet` mode (targeted constructs),
  - `sv_file` mode (full compilation units),
  - semantic-annotation-driven branch/value policies to synthesize legal SV.
- [ ] Enforce closed-loop convergence for SV:
  - generate -> parse -> semantic-validate -> coverage merge -> gap extraction -> targeted regeneration,
  - deterministic seed replay + shrinking for failing syntax/semantic samples.
- [ ] Add differential and integration hardening for Nexsim:
  - mismatch taxonomy against trusted references,
  - performance/memory budgets on realistic SV corpora,
  - embedding contract checks for Nexsim parser API usage.
- [ ] Promote SV gates into SOTA aggregate policy:
  - informational first,
  - required strict once syntax+semantic closure thresholds are green and stable.

### Phase Q (New): SystemVerilog Preprocessor Frontend Closure (Preprocessor-First)
Objective: deliver an executable, testable, deterministic preprocessor frontend so the main SV parser consumes preprocessed content under a defined contract.

- [x] Add `grammars/systemverilog_preprocessor.ebnf` as a dedicated grammar (separate from `systemverilog.ebnf`) and define directive-level syntax coverage baseline:
  - define/undef directives
  - include directives
  - ifdef/ifndef/elsif/else/endif directives
  - timescale, default_nettype, and celldefine flows
  - macro formal/actual argument forms and token-paste/stringize primitives as supported.
- [x] Add preprocessor parser + execution stage in Rust AST pipeline:
  - raw SV text -> preprocessor AST/events -> expanded SV text stream,
  - deterministic include/macro expansion policy,
  - source mapping metadata (expanded position -> original file/line/column) for diagnostics and Nexsim integration.
  - Progress (2026-02-27): added `rust/src/sv_preprocessor.rs` with deterministic execution for `define/undef/include/ifdef-family` + object/function macro expansion (`token-paste`/`stringize` baseline), include cycle/depth controls, structured event log output, and source-map metadata.
  - Progress (2026-02-27): wired AST-pipeline CLI mode `--preprocess-systemverilog` with include-dir/depth/redefine controls and optional JSON artifact emission (`--sv-source-map-json`, `--sv-event-log-json`).
- [x] Add `sv_preprocessor_quality_gate`:
  - deterministic replay across seeds,
  - coverage/gap loop for preprocessor grammar,
  - include/macro conditional-branch coverage metrics,
  - shrinking for failing preprocessability samples.
  - Progress (2026-02-27): gate script implemented (`rust/scripts/sv_preprocessor_quality_gate.sh`) with stage-0 deterministic replay checks, closed-loop stage progression invariants, key preprocessor-rule hit assertions, target-drive integrity checks, and deterministic coverage-guided fuzz replay verification.
  - Current behavior: parseability validation and parseability-failure shrink checks are auto-enabled when parser-registry support for `systemverilog_preprocessor` is available; until then, gate runs in coverage/gap deterministic mode and reports adapter absence explicitly.
- [ ] Add preprocessor semantic controls and validator contracts (annotation-driven where appropriate):
  - include path policy + depth budget,
  - macro redefinition policy,
  - conditional-compilation expression policy,
  - strict warning/error promotion for unsafe directive combinations.
- [ ] Add parser/stimuli integration contract:
  - `sv_stimuli_quality_gate` must run `preprocess -> parse_full -> semantic-validate`,
  - stimuli modes expanded with preprocess-aware generation (`sv_pp_snippet`, `sv_pp_file`),
  - closed-loop feedback tracks both preprocessor and parser coverage/gap convergence.
- [ ] Add differential hardening for preprocessor behavior against trusted references (where available) and publish mismatch taxonomy.
- [ ] Promote preprocessor gate policy:
  - informational first while grammar closes,
  - required strict before declaring Phase P (Nexsim SV parser closure) complete.

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
- 2026-02-27: Implemented Phase Q preprocessor execution stage in Rust AST pipeline (`sv_preprocessor` module + `ast_pipeline --preprocess-systemverilog` CLI mode), delivering deterministic include/macro expansion baseline and source-map/event metadata outputs.
- 2026-02-27: Implemented `sv_preprocessor_quality_gate` (script + Make target) and wired it into aggregate SOTA policy as informational (`run=1`, `strict=0`) for early Phase Q closure while parser-registry parseability support is pending.
- 2026-02-27: Added executable `grammars/systemverilog_preprocessor.ebnf` seed grammar and validated `EBNF -> JSON -> parser -> stimuli` on non-bootstrap Rust pipeline, closing Phase Q item for dedicated preprocessor grammar baseline.
- 2026-02-27: Added Phase Q (`SystemVerilog Preprocessor Frontend Closure`) and made it an explicit preprocessor-first prerequisite for Phase P closure, including dedicated grammar, preprocess execution stage, preprocess quality gate, preprocess-aware stimuli modes, and policy-promotion path.
- 2026-02-27: Added Phase P (`SOTA SystemVerilog Parser + Stimuli Semantic Closure`) to codify the Nexsim-targeted execution plan: syntax+semantic equal acceptance contract, annotation-driven SV stimuli synthesis, and mandatory closed-loop coverage/gap convergence.
- 2026-02-27: Added initial `grammars/systemverilog.ebnf` (IEEE 1800 markdown-seeded) and validated end-to-end (`EBNF -> JSON -> parser -> stimuli`), moving HDL readiness report row `systemverilog` from `not_ready` to `pass` while `vhdl` remains pending.
- 2026-02-27: Started Phase O / Pillar 5 kickoff by adding `hdl_frontend_readiness` + `hdl_frontend_gate` (script: `rust/scripts/hdl_frontend_readiness_gate.sh`) with explicit `systemverilog`/`vhdl` roster, report-mode `not_ready` handling for missing grammars, and strict-mode failure semantics for merge-safe progression.
- 2026-02-26: Promoted Perl-vs-Rust EBNF dual-run differential from informational/report mode to required strict aggregate SOTA policy check (`PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`) after strict gate validation passed on tracked grammars (`ebnf/json/regex`).
- 2026-02-26: Advanced Phase M parseability promotion beyond annotation grammars by adding an `ebnf` generated-parser adapter in `parser_registry`, promoting `ebnf` to required parseability in `ebnf_stimuli_contract.json`, and hardening `ebnf_stimuli_quality_gate` with explicit `generated/ebnf.rs` bootstrap + `ebnf_dual_run` rebuild for executable enforcement.
- 2026-02-22: Closed Phase L semantic typed-AST closure item after full aggregate validation (`annotation_typed_ast_gate`) with strict generated conversion for backend-validated named semantic directives, corpus-level generated semantic conversion contracts, and non-bootstrap E2E pass.
- 2026-02-22: Hardened non-bootstrap named semantic extraction policy: enforce generated parse-tree conversion for backend-validated named directives and keep compatibility fallback only for backend-rejected named payloads; added regression coverage and validated with `annotation_nonbootstrap_e2e_gate`.
- 2026-02-22: Closed Phase L return typed-AST closure item by adding full generated-pass return corpus parity proof in `UnifiedReturnAST`, and advanced semantic typed-AST closure with a full generated-pass corpus conversion contract test in `UnifiedSemanticAST` (entry/direct parity + canonical reconstruction invariants + comparable bootstrap checks).
- 2026-02-22: Completed Phase M parseability-promotion closure by adding `builtin_semantic_annotation` parser-registry parseability adapter aligned to builtin semantic parser behavior and promoting builtin semantic parseability to required in the non-annotation grammar-quality contract.
- 2026-02-22: Completed Phase N documentation closure by expanding `PGEN_USER_GUIDE.md` with in-memory-vs-module usage guidance, concrete embedding/replay command examples, deterministic seed compatibility rules, and publishing dedicated normative stimuli-module contract spec in `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`.
- 2026-02-22: Closed Phase N parity gate wiring by adding `stimuli_module_parity_gate` (contract-driven in-memory vs module parity checks over samples/coverage/gap), promoting it into aggregate SOTA required-check policy (`rust/config/sota_exit_policy.env` + `rust/scripts/sota_exit_gate.sh`), and retaining parity artifacts under `rust/target/stimuli_module_parity_gate` in CI aggregate uploads.
- 2026-02-22: Completed Phase N deterministic artifact contract closure by hardening `--generate-stimuli-module` output invariants (stable API version constant, deterministic default seed when omitted, non-optional entry-rule metadata), adding deterministic source regression tests in `ast_pipeline`, and explicitly marking opt-in stimuli-module generation while preserving default in-memory stimuli behavior.
- 2026-02-21: Started Phase N by adding explicit `ast_pipeline --generate-stimuli-module` mode (JSON and EBNF frontend inputs) that emits Rust module artifacts with embedded generated sample corpus and metadata constants.
- 2026-02-21: Completed Phase L proof-doctrine/full-contract gate closure by adding semantic full-contract slices (`semantic_runtime_contract_gate`, `semantic_ast_roundtrip_gate`, `semantic_full_contract_gate`), introducing `annotation_100_gate` (construct coverage + typed-AST + runtime-intent + determinism + differential parity), wiring `annotation_100_gate` into SOTA required-check policy, and hardening deterministic return object-field code emission in `ast_return_transform` to stabilize `fixed_point_gate`.
- 2026-02-21: Advanced Phase L SA-01 baseline by replacing generated semantic round-trip identity behavior with generated parse-tree conversion (`parse_generated_semantic_annotation`) in `test_runner`, adding conversion regression coverage, and validating through `semantic_usage_gate` + `annotation_contract_gate`.
- 2026-02-21: Closed remaining RA-01 generated round-trip bootstrap dependency by switching `GeneratedReturnAnnotationParser::round_trip` to generated parse-tree typed conversion (`parse_generated_return_annotation`) before canonical unparse.
- 2026-02-21: Advanced Phase L RA-01 from baseline to structural closure progress by replacing span-based generated return conversion with rule-aware parse-tree mapping in `UnifiedReturnAST`, aligning extraction-target and zero/signed-zero positional semantics with bootstrap parity, expanding generated conversion parity corpus tests, and broadening `return_runtime_semantics_gate` to run the full `generated_return_tree_to_typed_ast_` family.
- 2026-02-20: Advanced Phase L RA-04 gate hardening by adding explicit return gate slices (`return_runtime_semantics_gate`, `return_ast_roundtrip_gate`, `return_full_contract_gate`) and wiring `return_full_contract_gate` into `annotation_contract_gate`.
- 2026-02-20: Advanced Phase L RA-03 by removing generated return round-trip identity behavior in `test_runner` and switching to shared typed canonical unparse output (`unparse_return_ast`), validated by `return_parity_gate` with zero comparable mismatches.
- 2026-02-20: Advanced Phase L RA-02 runtime closure baseline by adding typed return identifier literal support and single-quoted string/object-key parsing parity in `UnifiedReturnAST`, plus exhaustive transformer/validator/test-runner normalization handling and regression coverage.
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
- 2026-02-20: Started SC-11 negative-case steering baseline by adding typed validator payload/coherence diagnostics (`W_SEM_INVALID_INVALID_CASE_PAYLOAD`, `W_SEM_INVALID_NEGATIVE_PAYLOAD`, `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE`), generated-parser expected-failure runtime hooks/events (`NegativeCaseEvent` + counters/accessors), and stimuli invalid/near-invalid mutation routing with semantic usage gate coverage.
- 2026-02-20: Started SC-12 determinism partitioning baseline by adding typed validator payload/coherence diagnostics (`W_SEM_INVALID_SEED_GROUP_PAYLOAD`, `W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD`, `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP`) plus deterministic stimuli seed partition routing per semantic group (`@seed_group/@deterministic_group`) with order-independence regression coverage.
- 2026-02-20: Promoted SC-12 to parser-side steering baseline by adding deterministic group-aware ordered OR branch evaluation offsets (group-key hashed partitioning), typed parser partition telemetry (`DeterministicPartitionEvent` + per-rule counters/accessors), and semantic usage gate coverage for generated parser hooks/order rotation.
- 2026-02-20: Hardened SC-12 parser embedder controls by adding runtime partition override surface (`DeterministicPartitionRuntimeMode` with `AnnotationDriven`/`ForceEnabled`/`ForceDisabled`), moving ordered-OR partition ordering to runtime-effective resolution, and wiring deterministic partition event emission to runtime-effective enable/group state.
- 2026-02-20: Started SC-04 token-class steering baseline by adding typed payload diagnostics (`W_SEM_INVALID_TOKEN_CLASS_PAYLOAD`, `W_SEM_INVALID_CHARSET_PAYLOAD`, `W_SEM_INVALID_PATTERN_PAYLOAD`), deterministic precedence contract warning (`W_SEM_TOKEN_STEERING_PRECEDENCE`), grammar-aware inactive steering warning (`W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`), and parser/stimuli runtime steering (`@pattern > @charset > @token_class`) with semantic usage gate coverage.
- 2026-02-20: Promoted SC-04 to Tier-4 by adding a dedicated gate (`sc04_contract_gate`) with explicit semantic token-steering contract slices (`semantic_annotation_sc04_contract`) and differential mismatch taxonomy parity checks, then wiring that gate into `annotation_contract_gate` so existing CI required checks enforce SC-04 contract closure.
- 2026-02-20: Hardened SC-03 to Tier-4 by adding a dedicated gate (`sc03_contract_gate`) with explicit directive-routing contract slices (`semantic_annotation_sc03_contract`), strict unknown-directive/strict-warning policy checks, transform/literal named-routing coverage, and differential mismatch taxonomy parity checks; gate wired into `annotation_contract_gate` so CI required checks enforce SC-03 routing/strictness closure.
- 2026-02-20: Added Phase L with a dedicated zero-compromise annotation closure plan (`PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`) targeting full return/semantic grammar support with objective proof gates.
- 2026-02-20: Strengthened Phase L with explicit annotation proof-doctrine requirements (construct coverage, typed-AST no-fallback, runtime intent conformance, determinism, and parity) as mandatory gate work.
- 2026-02-20: Implemented `annotation_stimuli_quality_gate` and wired it into `annotation_contract_gate`; gate now enforces a strict deterministic closed-loop for return/semantic annotation grammars (baseline parseability+coverage+gap, gap-priority generation, target-driven generation, and final no-regression gap recompute with stage-level artifact/metric invariants).
- 2026-02-20: Added Phase M non-annotation quality loop with contract-driven `make ebnf_stimuli_quality_gate` (strict `EBNF -> JSON`, parser generation, and closed-loop stimuli/coverage/gap invariants for tracked non-annotation grammars) and promoted it to required aggregate SOTA policy checks.
- 2026-02-20: Promoted SC-07 to Tier-4 by adding dedicated gate `sc07_contract_gate` (typed recovery payload/coherence validator contracts, parser/stimuli recovery runtime contracts, shared SC-07 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-09 to Tier-4 by adding dedicated gate `sc09_contract_gate` (typed relational payload/coherence validator contracts, parser/stimuli relational runtime contracts, shared SC-09 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-10 to Tier-4 by adding dedicated gate `sc10_contract_gate` (typed coverage-target/critical-path payload+coherence contracts, parser/stimuli coverage steering runtime contracts, shared SC-10 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-11 to Tier-4 by adding dedicated gate `sc11_contract_gate` (typed invalid-case/negative payload+coherence contracts, parser/stimuli negative-case runtime contracts, shared SC-11 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-12 to Tier-4 by adding dedicated gate `sc12_contract_gate` (typed seed-group/deterministic-group payload+coherence contracts, parser/stimuli deterministic partition runtime contracts, shared SC-12 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-05 to Tier-4 by adding dedicated gate `sc05_contract_gate` (typed priority/precedence/associativity payload contracts, parser/stimuli precedence+associativity runtime contracts, shared SC-05 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-08 to Tier-4 by adding dedicated gate `sc08_contract_gate` (typed range/enum/len/regex payload contracts, parser/stimuli value-domain runtime contracts, shared SC-08 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Promoted SC-06 to Tier-4 by adding dedicated gate `sc06_contract_gate` (typed branch-policy validator contracts, parser/stimuli branch-selection runtime contracts, weighted-probability determinism checks, shared SC-06 semantic contract suite, and differential taxonomy parity checks) and wiring it into `annotation_contract_gate` for CI enforcement.
- 2026-02-20: Added Phase N roadmap track for generated stimuli-module artifacts (`generated/<grammar>_stimuli.rs`) with explicit in-memory-vs-module parity gate requirements and embedding contract milestones.
