# PGEN

PGEN is a production-focused parser and stimuli generator platform.

## Project Objective
- Build **state-of-the-art, EBNF-driven parser/stimuli generation** for serious language tooling.
- Parser-construction doctrine:
  - every parser that counts as a PGEN deliverable shall be EBNF-backed,
  - there are no exceptions to this rule,
  - handwritten parsers may exist only as bootstrap/prototyping scaffolding and do not count as final closure.
- Annotation doctrine:
  - every generated parser returns an AST,
  - return annotations are the normative mechanism for shaping that returned AST,
  - semantic annotations are the normative mechanism for steering parser-generation behavior.
- Parser proof doctrine:
  - for a deliverable grammar `grammars/foolang.ebnf`, closure expects a generated parser path (`generated/foolang_parser.rs`) plus a stimuli path,
  - that stimuli path may be the default in-memory generator, a generated module artifact (`generated/foolang_stimuli.rs`), or both,
  - when both stimuli forms exist, parity between them is part of the contract,
  - parser closure requires objective roundtrip and coverage proof for both parsing and stimuli generation rather than narrative confidence,
  - this doctrine applies to any PGEN EBNF-based parser family with no exception: SystemVerilog, VHDL, regex, annotation grammars, Phase S parser families, and future grammar families are all judged against the same professional-grade closure standard,
  - the live tracker differs by how much of that common proof doctrine has been landed for a given parser family, not by using different quality bars for different grammars.
- Support advanced **return annotations** and **semantic annotations** with contract-grade validation.
- Deliver parser/stimuli quality via deterministic gates, coverage/gap analysis, and closed-loop replay.
- Treat parser quality as the product:
  - generated parsers must be correct, fast, accurate, predictable, observable, and trustworthy in real systems.
- North-star trust goal:
  - make PGEN the de facto go-to platform for parsers because projects can trust it,
  - make PGEN sign-off-grade when parsing correctness materially affects downstream flows.
- Primary near-term integration targets:
  - **Nexsim** (SystemVerilog + VHDL parsing)
  - **RGX** (regex parsing)

## Canonical Flow
- `grammars/foolang.ebnf -> raw_ast/json -> generated/foolang_parser.rs`
- `grammars/foolang.ebnf -> in-memory stimuli and/or generated/foolang_stimuli.rs`
- Rust-native EBNF frontend now also supports direct `raw_ast` export:
  - `ast_pipeline INPUT.ebnf --emit-raw-ast-json RAW.json`
- Annotation parsers (`return_annotation_parser`, `semantic_annotation_parser`) are generated with bootstrap mode only.
- `grammars/builtin_return_annotation.ebnf` and `grammars/builtin_semantic_annotation.ebnf` are the bootstrap-safe annotation grammar contracts used for that bootstrap generation path, so the annotation parsers can be generated without depending on themselves.
- All other grammars use the non-bootstrap path.
- `grammars/return_annotation.ebnf` with `generated/return_annotation_parser.rs` defines the supported AST-shaping language for parser return values.
- `grammars/semantic_annotation.ebnf` with `generated/semantic_annotation_parser.rs` defines the supported steering language for parser-generation behavior.
- `make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate` is the required closed-loop proof surface for annotation stimuli quality, including the return-annotation generator/parser loop.
- `make -C rust SHELL=/bin/bash return_annotation_support_gate` is the focused aggregate proof surface for return-annotation closure in the Rust AST pipeline; it now includes the auto-derived `return_annotation_exhaustiveness_gate` (grammar-driven coverage closure, stimuli-module parity, and generated-parse-tree to typed-AST audit) and is the formal `Done` gate for the currently tracked return-annotation claim.
- In general, PGEN supports two stimuli-delivery modes for a grammar:
  - default in-memory generation via `--generate-stimuli`,
  - optional generated module artifacts via `--generate-stimuli-module` (for example `generated/foolang_stimuli.rs`).
- For serious parser closure claims, the expected evidence is:
  - EBNF-backed parser generation,
  - return-AST shaping through return annotations,
  - parser/stimuli roundtrip proof,
  - parser coverage proof,
  - stimuli-generation coverage/gap proof,
  - and repeatable machine-checkable gates behind every claim.
  - This is the repository-wide closure doctrine for any PGEN EBNF-based parser, not an SV-only or annotation-only rule.

## Fast Ramp-Up (Read In This Order)
1. `README.md` (this file)
2. `QUICKSTART_AI_ONBOARDING.md`
3. `PGEN_USER_GUIDE.md`
4. `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
5. `LIVE_ACHIEVEMENT_STATUS.md`
6. `PGEN_ANNOTATION_NORMATIVE_SPEC.md`
7. `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
8. `CHANGES.md`
9. `DEVELOPMENT_NOTES.md`
10. `MEMORY.md`
11. `COMMIT.md`

## Key Project Paths
- `grammars/`: EBNF sources (`*.ebnf`)
- `grammars/builtin_return_annotation.ebnf`, `grammars/builtin_semantic_annotation.ebnf`: bootstrap-safe annotation grammar contracts that break the annotation-parser chicken-and-egg cycle
- `generated/`: version-controlled canonical generated artifacts used by compile-time includes and clean-checkout gates
- `rust/target/generated_logs/`: scratch generation/debug logs kept out of `generated/`
- `rust/src/`: Rust AST pipeline, generators, parser registry, embedding API
- `rtl_const_expr/`: standalone constant-expression parser/evaluator bootstrap baseline crate for planned RTL frontend/elaboration work, including dotted and package-qualified (`pkg::NAME`) identifier lookup; it exists because RTLSyn needs deterministic parameter/width/generate evaluation before elaboration can be trusted, and Phase S closure still requires a tracked EBNF-backed parser path
- `rtl_frontend/`: initial synthesizable-RTL frontend bootstrap baseline crate wired to `rtl_const_expr` for module/instance parsing, typed port actuals (including member-path/expression/repetition forms), unpacked-array port/net declarations, struct-aware validation through indexed unpacked-array elements, enum and union data types with typedef/import visibility, builtin integral atom types (`byte`, `shortint`, `longint`) in declarations and enum base-width handling, `always_ff` edge-event controls and `always_latch` procedural blocks in addition to `always_comb` / `always @(*)`, typed assignment targets for `assign` and procedural statements (including signal/member/select/part-select/concatenation forms), structured assignment values (including signal/member/select/concat/repeat forms), elaboration-time procedural validation for known identifiers plus `always_ff` nonblocking-assignment policy, packed-union width-coherence validation, instance-array expansion, inline aggregate-aware member validation, file-scope/module-local/package typedef-backed named types, package-backed constant declarations plus package-qualified/body-import/header-import constant visibility, and first-pass elaboration helpers; it exists because RTLSyn needs a trusted synthesis-facing RTL/elaboration boundary before Liberty/SDC and downstream synthesis logic can bind meaningfully, and Phase S closure still requires a tracked EBNF-backed parser path
- `rust/build.rs`: compile-time generated-parser include path resolver; emits relative `include!(env!(...))` paths from `rust/src/` so clean checkouts and relocated worktrees do not depend on absolute filesystem paths
- `rust/config/branch_protection_policy.json`: tracked minimum branch-protection required-check contract
- `rust/scripts/`: executable quality gates and policy runners
- `rust/test_data/grammar_quality/`: gate contracts, corpora, deterministic case manifests
- `rust/docs/`: Rust-specific architecture/API/test docs
- `tools/`: conversion/extraction and support workflows
- `perl/`: legacy/frontend EBNF-to-JSON path (`ebnf_to_json.pl`) still used in hybrid flow
- `docs/systemverilog/2017`, `docs/systemverilog/2023`: SV LRM conversion workspaces
- `docs/vhdl/2019`: VHDL LRM conversion workspace
- `docs/verilog/2005`: Verilog LRM conversion workspace
- `grammars/verilog_2005_lrm_extracted.ebnf`: canonical extracted Verilog 2005 grammar snapshot from the tracked LRM workspace
- `grammars/systemverilog.ebnf`: active flattened profile-aware full-SV grammar synthesized from the IEEE 1800-2017/2023 markdown workspaces (`sv_2017`, `sv_2023`)
- `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`: full extracted SV EBNF snapshots from the versioned markdown workspaces
- `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`: profiled synthesis artifacts retained for regeneration traceability
- `docs/systemverilog/profiled_generation_report.json`: structured report for staged dual-LRM profile synthesis
- `tests/`: test how-to and test guides

## Standard Commands
- Aggregate policy gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
- Branch-protection contract gate:
  - `make -C rust SHELL=/bin/bash branch_protection_contract_gate`
- Local workflow parity gate:
  - `make -C rust SHELL=/bin/bash ci_workflow_local_gate`
  - focused replay example:
    - `PGEN_CI_WORKFLOW_LOCAL_FILTER=annotation-contract-gate make -C rust SHELL=/bin/bash ci_workflow_local_gate`
- SV quality gate:
  - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
  - bounded replay rerun example:
    - `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=100 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
- VHDL quality gate:
  - `make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate`
- VHDL strict-promotion trials:
  - `make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate`
- EBNF dual-run gate:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
- Return-annotation support gate:
  - `make -C rust SHELL=/bin/bash return_annotation_support_gate`
- Stimuli module parity gate:
  - `make -C rust SHELL=/bin/bash stimuli_module_parity_gate`
- EBNF frontend readiness (Rust path):
  - `PGEN_EBNF_FRONTEND_IMPL=rust make -C rust SHELL=/bin/bash ebnf_frontend_readiness`
- EBNF closed-loop quality (Rust path):
  - `PGEN_EBNF_FRONTEND_IMPL=rust PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh`

## Documentation Status
- Current authoritative docs for the active Rust-first platform:
  - `README.md`
  - `PGEN_USER_GUIDE.md`
  - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Historical/reference docs are still tracked for context, but some describe superseded workflows or earlier project phases.
- In particular, treat these as archival unless they are explicitly refreshed:
  - `CURRENT_STATUS.md`
  - `PROJECT_OVERVIEW.md`
  - `QUICKSTART_AI_ONBOARDING.md`
  - `rust/docs/TECHNICAL_ARCHITECTURE.md`
  - `rust/docs/CLI_REFERENCE.md`
- The complete markdown index below is a repository navigation index, not a claim that every listed document is equally current.
- Commit-workflow continuity rule:
  - `COMMIT.md` is binding operational policy for post-task commits,
  - post-commit user-facing reports must include the commit ID, exact commit message, the list of tracked files included in the commit, and the current live-status snapshot.

## Documentation Structure
- Project governance and status:
  - `PROJECT_OVERVIEW.md`, `CURRENT_STATUS.md`, `PGEN_RELEASE_POLICY.md`, `WARP.md`
- Core contracts and roadmaps:
  - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, `PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`, `PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`, `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, `SV_GRAMMAR_COVERAGE_MATRIX.md`
- Operational continuity:
  - `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`
- User/developer onboarding:
  - `PGEN_USER_GUIDE.md`, `QUICKSTART_AI_ONBOARDING.md`, `IMPLEMENTATION_GUIDE.md`, `STRESS_TEST_STANDARDIZATION.md`

## Complete Markdown Index (Tracked)
The list below is the complete set of tracked markdown files and is intended to keep `README.md` as the single navigation entrypoint.
- `CHANGES.md`
- `COMMIT.md`
- `CURRENT_STATUS.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/api_interfaces.md`
- `docs/AST_BASED_GENERATOR.md`
- `docs/AST_GENERATOR_ARCHITECTURE.md`
- `docs/AST_GENERATOR_MIGRATION.md`
- `docs/AST_TRANSFORM_REFACTOR_PLAN.md`
- `docs/ast_transformation_pipeline.md`
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md`
- `docs/BOOTSTRAP_SYSTEM_COMPLETE.md`
- `docs/CLEANUP_SUMMARY.md`
- `docs/COMPLETE_AST_TRANSFORMATION_PIPELINE.md`
- `docs/DEBUGGING_STARTUP_GUIDE.md`
- `docs/DEVELOPMENT_NOTES.md`
- `docs/EBNF_GENERATOR_ARCHITECTURE.md`
- `docs/EBNF_GRAMMAR_RULES.md`
- `docs/EBNF_IMPROVEMENT_ROADMAP.md`
- `docs/EBNF_INCLUDE_SYSTEM.md`
- `docs/EBNF_PARSER_GENERATOR_GUIDE.md`
- `docs/EBNF_PARSER_GENERATOR.md`
- `docs/EBNF_QUICK_REFERENCE.md`
- `docs/ERROR_REPORTING_GUIDE.md`
- `docs/fully_featured_return_annotation_parsers_status.md`
- `docs/GROUPED_QUANTIFIER_DOCUMENTATION_INDEX.md`
- `docs/GROUPED_QUANTIFIER_FIXES_SUMMARY.md`
- `docs/GROUPING_QUANTIFIERS_ANALYSIS.md`
- `docs/HDL_GRAMMAR_VALIDATION_REPORT.md`
- `docs/HYBRID_AST_IMPLEMENTATION.md`
- `docs/implementation_complete.md`
- `docs/json_schemas.md`
- `docs/julia_parser_gen.md`
- `docs/LINKEDSPEC_DEEP_UNDERSTANDING.md`
- `docs/LINKEDSPEC_IMPROVEMENTS.md`
- `docs/multi_language_architecture.md`
- `docs/MULTI_LANGUAGE_PARSER_VISION.md`
- `docs/parser_architecture_evolution.md`
- `docs/PARSER_REGENERATION_SUMMARY.md`
- `docs/PERFORMANCE_GUIDE.md`
- `docs/PROJECT_STATUS_REPORT.md`
- `docs/python_ast_pipeline.md`
- `docs/python_syntactic_data_generator.md`
- `docs/QUANTIFIED_SEQUENCE_SERIALIZATION_FIX.md`
- `docs/RETURN_ANNOTATION_PARSER.md`
- `docs/return_annotation_self_hosting.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/round_trip_testing_ideas.md`
- `docs/RUST_AST_SEMANTIC_ANNOTATIONS.md`
- `docs/rust_parser_gen.md`
- `docs/SEMANTIC_ANNOTATIONS_ANALYSIS.md`
- `docs/STRING_GENERATOR_FEATURES_TO_PORT.md`
- `docs/SYNTACTIC_DATA_GENERATOR.md`
- `docs/systemverilog/README.md`
- `docs/TEST_INFRASTRUCTURE.md`
- `docs/test_stability_plan.md`
- `docs/tools.md`
- `docs/ULTIMATE_DOT_NOTATION_DOCS.md`
- `docs/ultimate_return_annotation_parser_status.md`
- `docs/universal_return_annotation_system.md`
- `docs/verilog/README.md`
- `docs/vhdl/README.md`
- `IMPLEMENTATION_GUIDE.md`
- `MEMORY.md`
- `PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`
- `PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `PGEN_RELEASE_POLICY.md`
- `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `PGEN_USER_GUIDE.md`
- `PROJECT_OVERVIEW.md`
- `QUICKSTART_AI_ONBOARDING.md`
- `README.md`
- `REGEX_BOOTSTRAP_ARCHITECTURE.md`
- `rust/BRANCH_RETURN_ANNOTATIONS.md`
- `rust/DEBUG_IMPLEMENTATION.md`
- `rust/docs/CLI_REFERENCE.md`
- `rust/docs/DEVELOPMENT_GUIDE.md`
- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `rust/docs/TECHNICAL_ARCHITECTURE.md`
- `rust/docs/TEST_AUTOMATION.md`
- `rust/LOG_FILES_README.md`
- `rust/RETURN_ANNOTATION_PIPELINE.md`
- `rust/RETURN_ANNOTATION_STATUS.md`
- `STRESS_TEST_STANDARDIZATION.md`
- `SV_GRAMMAR_COVERAGE_MATRIX.md`
- `tests/GENERATE_TEST_INPUT.md`
- `tests/README.md`
- `tests/TEST_GUIDE.md`
- `tools/LRM_CONVERSION_WORKFLOW.md`
- `WARP.md`
- `zig/zig-0.15.1-arraylist-changes.md`
