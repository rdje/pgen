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
  - **PNR** (staged LEF / Liberty / DEF / Verilog structural netlist / SDC / SPEF parser-family demand captured as a downstream contract)

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
- `make -C rust SHELL=/bin/bash annotation_contract_gate` is the aggregate annotation contract spine for validator coverage, built-in/shared annotation suites, SC semantic contract slices, aggregate semantic/return contract gates, and annotation robustness/stimuli verification.
- `make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate` is the required closed-loop proof surface for annotation stimuli quality, including the return-annotation generator/parser loop.
- `make -C rust SHELL=/bin/bash semantic_full_contract_gate` is the focused aggregate proof surface for semantic annotation runtime, round-trip, and comparable differential-regression evidence.
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
2. `docs/book/` (`mdBook` live mastery surface)
3. `QUICKSTART_AI_ONBOARDING.md`
4. `PGEN_USER_GUIDE.md`
5. `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
6. `LIVE_ACHIEVEMENT_STATUS.md`
7. `docs/reference/RUST_CODEBASE_ANALYSIS.md`
8. `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
9. `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
10. `CHANGES.md`
11. `DEVELOPMENT_NOTES.md`
12. `MEMORY.md`
13. `COMMIT.md`

## Key Project Paths
- `grammars/`: EBNF sources (`*.ebnf`)
- `grammars/builtin_return_annotation.ebnf`, `grammars/builtin_semantic_annotation.ebnf`: bootstrap-safe annotation grammar contracts that break the annotation-parser chicken-and-egg cycle
- `generated/`: version-controlled canonical generated artifacts used by compile-time includes and clean-checkout gates
- `rust/target/generated_logs/`: scratch generation/debug logs kept out of `generated/`
- `rust/src/`: Rust AST pipeline, generators, parser registry, embedding API
- `rtl_const_expr/`: standalone constant-expression parser/evaluator bootstrap baseline crate for planned RTL frontend/elaboration work, including dotted and package-qualified (`pkg::NAME`) identifier lookup; it is now paired with tracked grammar `grammars/rtl_const_expr.ebnf` and generated parser `generated/rtl_const_expr_parser.rs` because RTLSyn needs deterministic parameter/width/generate evaluation before elaboration can be trusted
- `rtl_frontend/`: initial synthesizable-RTL frontend bootstrap baseline crate wired to `rtl_const_expr` for module/instance parsing, typed port actuals (including member-path/expression/repetition forms), unpacked-array port/net declarations, struct-aware validation through indexed unpacked-array elements, enum and union data types with typedef/import visibility, builtin integral atom types (`byte`, `shortint`, `longint`) in declarations and enum base-width handling, `always_ff` edge-event controls and `always_latch` procedural blocks in addition to `always_comb` / `always @(*)`, typed assignment targets for `assign` and procedural statements (including signal/member/select/part-select/concatenation forms), structured assignment values (including signal/member/select/concat/repeat forms), elaboration-time procedural validation for known identifiers plus `always_ff` nonblocking-assignment policy, packed-union width-coherence validation, instance-array expansion, inline aggregate-aware member validation, file-scope/module-local/package typedef-backed named types, package-backed constant declarations plus package-qualified/body-import/header-import constant visibility, and first-pass elaboration helpers; it is now also paired with tracked bootstrap grammar `grammars/rtl_frontend.ebnf`, generated artifacts `generated/rtl_frontend_parser.rs` / `generated/rtl_frontend.json`, curated generated-contract manifest `rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json`, and gate `make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate`, whose retained sample set now reaches empty no-port multi-module declarations, module-local parameter/localparam items, generate-if/else local net declarations, generate-for local net declarations, generate-if/else dataflow, mixed procedural/dataflow member-path cases, single- and multi-module file-scope, local, package-qualified, wildcard-imported, and named-imported struct typedef surfaces, local enum/union typedef surfaces, inline struct typed nets, handwritten-baseline `byte` union field-name surfaces, builtin integral typed nets and inline enum base-type forms, header-imported struct/enum/union typedef ports, package-qualified/header-wildcard-imported/module-named-imported constant parameter and range flows, procedural/dataflow ternary assignment values plus labeled parameter-expression `always_comb` procedural blocks, packed multi-net declarations, scalar `always @(*)` if/else blocks, scalar nonblocking `always_latch` blocks plus syntax-only unknown-body-identifier latch parsing, isolated `always_ff` struct-member bit-select nonblocking targets, isolated `always_ff` struct-member concatenation values, syntax-only `always_ff` event-control identifier parse surfaces, isolated `always_comb` struct-member concatenated assignment targets, syntax-only unknown-member continuous assignment target/value and concatenated-target parse surfaces, isolated continuous struct-member bit-select assignment targets, isolated continuous struct-member concatenated assignment targets, isolated continuous struct-member concatenation values, and richer plain-`always @(*)`, `always_latch`, sequential/procedural/dataflow ranged and concatenated assignment targets, generated isolated scalar and ranged/member `always_ff` blocking-assignment rejections, `always_latch` event-control rejection, ranged/concatenated assignment-target near-miss rejects including lane-local plain-`always @(*)` / `always_latch` ranged/member and concatenated-target rejects, scalar named-parameter-override/named-port module instantiations, scalar wildcard-port module instantiations, named-port, parameterized named-port, and wildcard-port instance-array syntax, named-port union-member actuals plus syntax-only unknown union-member actuals, named-port bit-select/concatenation actuals, named-port member bit-select/repetition actuals, ordered parameter/port actuals with repeat-concatenation values, deeper ordered actuals with comma-bearing repeat-concatenation member ranges, ordered/named parameter overrides and ordered/named port actuals with ternary/binary expressions including named-port member-path ternaries, named parameter overrides and named port actuals with repeat-concatenation range expressions, ternary and repeat/list near-miss rejects, homogeneous named/ordered override and port-list rejects, and earlier generate/dataflow lanes, while the remaining Phase S work is broader parity/proof closure against the handwritten baseline
- Current `rtl_frontend` generated-contract note: the retained sample set now also includes syntax-only unindexed unpacked-array, known/unknown typedef-backed struct-member, unknown parent-identifier named-port actuals, and inline, builtin-integral, plus typedef-backed packed-union field-width mismatch declarations, proving parser acceptance for `child u_child (.a(cfgs.data), .y(y));`, `child u_child (.a(cfg.data), .y(y));`, `child u_child (.a(cfg.missing), .y(y));`, `child u_child (.a(missing_signal), .y(y));`, `union packed { logic [7:0] data; logic [15:0] word; } payload;`, `union packed { byte data; shortint word; } payload;`, and `typedef union packed { logic [7:0] data; logic [15:0] word; } payload_t; payload_t payload;` while leaving corresponding semantic acceptance/rejection decisions to elaboration.
- `rust/build.rs`: compile-time generated-parser include path resolver; emits relative `include!(env!(...))` paths from `rust/src/` so clean checkouts and relocated worktrees do not depend on absolute filesystem paths
- `rust/config/branch_protection_policy.json`: tracked minimum branch-protection required-check contract
- `rust/scripts/`: executable quality gates and policy runners
- `rust/test_data/grammar_quality/`: gate contracts, corpora, deterministic case manifests
- `rust/docs/`: Rust-specific architecture/API/test docs
- `docs/contracts/`: downstream parser integration contracts, issue-reporting protocol, and released-parser bug ledger
- `docs/reference/`: normative specs, matrices, closure roadmaps, release policy, and other maintained deep-reference docs
- `regex_corpus_bundle/`: PCRE2-first regex corpus acquisition/inventory starter for future regex hardening; keeps immutable upstream snapshots separate from normalized corpus/oracle outputs, with maintained gates `make -C rust regex_corpus_bundle_contract_gate`, `make -C rust regex_pcre2_textsafe_corpus_gate`, and `make -C rust regex_pcre2_compile_oracle_gate`
- `tools/`: conversion/extraction and support workflows
- `perl/`: legacy/frontend EBNF-to-JSON path (`ebnf_to_json.pl`) still used in hybrid flow
- `docs/systemverilog/2017`, `docs/systemverilog/2023`: SV LRM conversion workspaces
- `docs/vhdl/2019`: VHDL LRM conversion workspace
- `docs/verilog/2005`: Verilog LRM conversion workspace
- `grammars/verilog_2005_lrm_extracted.ebnf`: canonical extracted Verilog 2005 grammar snapshot from the tracked LRM workspace
- `grammars/systemverilog.ebnf`: active flattened profile-aware full-SV grammar synthesized from the IEEE 1800-2017/2023 markdown workspaces (`sv_2017`, `sv_2023`)
- `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`: full extracted SV EBNF snapshots from the versioned markdown workspaces
- `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`: profiled synthesis artifacts retained for regeneration traceability
- `grammars/rtl_const_expr.ebnf`: tracked Phase S constant-expression grammar already paired with `generated/rtl_const_expr_parser.rs`
- `grammars/rtl_frontend.ebnf`: tracked bootstrap EBNF for the current RTLSyn-facing synthesizable RTL subset, now paired with generated parser artifacts, registry wiring, and a curated generated-contract gate; next step is broader parity/proof closure against the handwritten baseline
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
- mdBook docs gate:
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate`
- `rtl_frontend` generated contract gate:
  - `make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate`
  - focused workflow-parity replay example:
    - `PGEN_CI_WORKFLOW_LOCAL_FILTER=rtl-frontend-generated-contract-gate make -C rust SHELL=/bin/bash ci_workflow_local_gate`
- Cross-family stimuli platform gate:
  - `make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate`
  - bounded shared replay over:
    - regex via the regex-only EBNF stimuli contract
    - VHDL via bounded closed-loop replay
    - SystemVerilog via bounded single-profile (`2017`) `sv_parseable_file` closed-loop replay
  - emits:
    - `rust/target/stimuli_cross_family_platform_gate/summary.txt`
    - `rust/target/stimuli_cross_family_platform_gate/summary.json`
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
- Regex external hardening lanes:
  - `make -C rust regex_corpus_bundle_contract_gate`
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`

## Documentation Book
- The curated live book source is under:
  - `docs/book/`
- Build it locally with:
  - `mdbook build docs/book`
- Serve it locally with live reload:
  - `mdbook serve docs/book --open`
- Gate it with the repo-standard wrapper:
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate`
- Intent:
  - the book is the primary public documentation surface for users and developers,
  - the book itself should explain the documentation split between public chapters, deep reference/contracts, and internal continuity docs,
  - the book should grow until every important aspect of PGEN is documented there with rationale and transparency,
  - continuity docs are internal session/continuity surfaces,
  - contracts/reference docs remain the deep authoritative detail behind the book.

## Documentation Status
- Current authoritative docs for the active Rust-first platform:
  - `README.md`
  - `docs/book/`
  - `PGEN_USER_GUIDE.md`
  - `QUICKSTART_AI_ONBOARDING.md`
  - `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
  - `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  - `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
  - `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Historical/reference docs are still tracked for context, but some describe superseded workflows or earlier project phases.
- In particular, treat these as archival unless they are explicitly refreshed:
  - `rust/docs/TECHNICAL_ARCHITECTURE.md`
  - `rust/docs/CLI_REFERENCE.md`
- The complete markdown index below is a repository navigation index, not a claim that every listed document is equally current.
- Commit-workflow continuity rule:
  - `COMMIT.md` is binding operational policy for post-task commits,
  - post-commit user-facing reports must include the commit ID, exact commit message, the list of tracked files included in the commit, and the current live-status snapshot.

## Documentation Structure
- Curated live mastery book:
  - `docs/book/`
- Project governance, release policy, and live status:
  - `docs/reference/PGEN_RELEASE_POLICY.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`
- Rust architecture/state assessment:
  - `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- Core contracts and roadmaps:
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, `docs/reference/PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`, `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`, `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, `docs/reference/SV_GRAMMAR_COVERAGE_MATRIX.md`
- Downstream parser integration contracts:
  - `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`, `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`, `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`, `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
- Regex corpus acquisition and hardening:
  - `regex_corpus_bundle/README.md`, `regex_corpus_bundle/docs/regex_corpus_plan.md`, `regex_corpus_bundle/corpus/pcre2/invalid/README.md`, `regex_corpus_bundle/corpus/pcre2/quarantine/README.md`, `regex_corpus_bundle/oracle/pcre2/README.md`
- Operational continuity:
  - `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`
- User/developer onboarding:
  - `SESSION_BOOTSTRAP.md`, `README.md`, `QUICKSTART_AI_ONBOARDING.md`, `PGEN_USER_GUIDE.md`, `docs/reference/STRESS_TEST_STANDARDIZATION.md`

## Active Markdown Index
The list below is the current high-signal markdown surface for active work. A 2026-04-06 audit found that most top-level `docs/*.md` files are legacy implementation notes, historical status snapshots, or duplicate design writeups and should not be treated as equal-priority sources of truth.
- `CHANGES.md`
- `COMMIT.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `MEMORY.md`
- `PGEN_USER_GUIDE.md`
- `QUICKSTART_AI_ONBOARDING.md`
- `SESSION_BOOTSTRAP.md`
- `docs/book/book.toml`
- `docs/book/src/SUMMARY.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_RELEASE_POLICY.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `regex_corpus_bundle/README.md`
- `regex_corpus_bundle/docs/regex_corpus_plan.md`
- `regex_corpus_bundle/corpus/pcre2/invalid/README.md`
- `regex_corpus_bundle/corpus/pcre2/quarantine/README.md`
- `regex_corpus_bundle/oracle/pcre2/README.md`
- `docs/AST_GENERATOR_ARCHITECTURE.md`
- `docs/ast_transformation_pipeline.md`
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md`
- `docs/EBNF_INCLUDE_SYSTEM.md`
- `docs/parser_architecture_evolution.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/systemverilog/README.md`
- `docs/TEST_INFRASTRUCTURE.md`
- `docs/verilog/README.md`
- `docs/vhdl/README.md`

The top-level `docs/*.md` surface has now been pruned down to the maintained active reference set. The full audit trail and removal rationale remain recorded in `DEVELOPMENT_NOTES.md`.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now audits the tracked top-level `docs/*.md` allowlist so this surface does not silently drift back upward.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits the tracked `docs/contracts/*.md` and `docs/reference/*.md` allowlists so the curated contract/reference buckets do not silently drift.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits the curated `docs/book/` surface and replays the tracked `mdbook-docs-gate` workflow command so the live book stays buildable.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits active operator/reference docs for stale pre-rehome path mentions, so live docs keep pointing at the canonical `docs/contracts/...` and `docs/reference/...` locations.

Root markdown policy note:
- the repository root should be reserved for entrypoint docs, live continuity docs, and tool/session-control docs
- tool-specific editor/assistant docs that no longer serve the active workflow should be removed rather than kept as root clutter
- the parser integration contract surface now lives under `docs/contracts/` instead of consuming repo-root markdown slots
- the maintained spec / matrix / policy reference surface now also lives under `docs/reference/` instead of consuming repo-root markdown slots
- the active roadmap and the live Rust architecture/state assessment now also live under `docs/reference/` instead of consuming repo-root markdown slots
- stale historical root overview/status/guidance docs have now been removed instead of being kept as dead navigation noise
- the remaining root markdown set is now the intentionally minimal entrypoint / continuity / active-operator surface, while deep-reference docs like the roadmap, Rust analysis, and regex bootstrap architecture live under `docs/reference/`
- a separate root `*.md` audit/classification now also lives in `DEVELOPMENT_NOTES.md`
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now audits the tracked root markdown allowlist so this surface does not silently drift

Read SESSION_BOOTSTRAP.md and start from there.
