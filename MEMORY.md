# MEMORY.md

Last updated: 2026-02-27 (+0100, task: phase-p-sv-semantic-closure-mode-profile)

## Purpose
Live session-continuity file for fast crash recovery and AI handoff.

Use this file to resume work without replaying full chat history.

## Resume Checklist (Read In Order)
1. `git status -sb`
2. Read latest entries in:
   - `CHANGES.md`
   - `DEVELOPMENT_NOTES.md`
   - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
3. Confirm current policy in:
   - `rust/config/sota_exit_policy.env`
4. Confirm untracked-policy files still untracked:
   - `git_message_brief.txt`
   - `questions_keep_untracked.txt`
5. If generated artifacts are needed, regenerate; do not assume they are committed.
6. Continue with highest-priority pending task (see "Next Likely Tasks").

## Current Technical Snapshot
- Branch: `main` (ahead of `origin/main`; run `git status -sb` for exact count).
- Worktree: dirty (pending commit workflow for SV semantic-closure mode/profile increment; run `git status -sb`).
- Latest commit: see tail entry in "Session Git History (Hash + Message)".
- SOTA policy status:
  - strict EBNF readiness required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`
  - strict EBNF dual-run required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
- Non-annotation parseability contract:
  - `ebnf` is now `require_parseability=true` (with `ebnf_dual_run` adapter path).

## Session Git History (Hash + Message)
- Scope used for continuity tracking: `origin/main..HEAD`
- Commit count at last refresh (before current uncommitted changes): `175`
- Refresh command:
  - `git log --oneline --reverse origin/main..HEAD`
<!-- SESSION_GIT_HISTORY_BEGIN -->
- e7e8ee1 Add deterministic initial replay equivalence checks to SV closed-loop gate
- fd7c349 Align SV stimuli sample-stage order with Phase Q parser/stimuli contract
- f5c2928 Track preprocess convergence debt in SV stimuli closed-loop gate
- 5115a23 Promote SV preprocessor quality gate to required aggregate policy
- 87e6a95 Add trusted-reference mismatch taxonomy to SV preprocessor quality gate
- d4be882 Add preprocess-aware SV stimuli modes to phase-Q quality contract
- 86c78cf Wire VHDL stimuli quality gate into aggregate SOTA policy
- d757fbe Add dedicated VHDL closed-loop stimuli quality gate for Nexsim hardening
- a997a78 Add mode-level recovery steering to SV stimuli gate (contract v10)
- 1b10f2a Add mode-level semantic override profiles to SV stimuli gate (contract v9)
- 53d7881 Extend SV context legality baseline with generate-loop genvar checks
- d840b68 Add SV semantic baseline port-binding legality toggle (contract v8)
- a8a99b3 Harden SV closed-loop gate with deterministic failure replay/shrinking (contract v7)
- f0a7133 Add SV stimuli mode profiles (sv_file/sv_snippet) to quality gate
- 825c3dd Wire Phase P semantic-closure validator toggles into sv_stimuli_quality_gate (contract v5)
- 3966b88 Add deterministic sv_syntax_closure_gate and close Phase P syntax burn-down loop
- e86f217 Promote SV stimuli gate to closed-loop baseline and freeze systemverilog_core_v0 v4
- f465acf Nexsim HDL closure: parser replay gate hardening and API convenience wrappers
- fb4dc4e Harden embedding API ergonomics for zero-friction Rust and FFI use
- 34d1f4f Add parser-profile embedding API scaffold for Nexsim integration
- 473dbe4 hdl: add executable vhdl seed grammar and close strict readiness gap
- 1f6a89f policy: promote aggregate hdl readiness to required strict mode
- cb67aab sv gate: extend semantic baseline with contract-driven structural checks (v2)
- 5dcf40a tools: add reusable IEEE LRM conversion pipeline for SV/VHDL docs workspaces
- 671ed6b Add SV dual-LRM profile scaffold and roadmap/API alignment
- ae177e2 fix: implement proper round-trip testing with correct normalization
- 9cd9e94 fix: convert remaining JSON files to correct round-trip format
- c2f70b4 fix: implement true round-trip testing with proper unparsing
- 091aea0 docs: update all documentation with round-trip testing framework
- 9816362 Fix Rust compilation errors, migrate to directory-based module structure, resolve type visibility issues, and enable test runner functionality.
- d749292 feat: Implement comprehensive parser logging infrastructure
- a9a9b54 Removed *_parser.rs from the exclusion list
- a825b1b feat: Complete AST-based code generator restoration - 31K+ lines of generated parsers working
- 0aeb319 Removed test-* targets except test-parser
- aed9eaf Remove obsolete test modules after test-* targets cleanup
- e897a3c Restore test_discovery module - file exists and is functional
- 0a45639 Clean up lib.rs - remove all obsolete commented-out module declarations
- af0201d Remove obsolete universal_test_runner.rs - not used in active codebase
- b32d719 Remove obsolete test generator files from dismantled test automation system
- 6658158 Cleaned up
- c4f06b8 Remove obsolete run_comprehensive_stress_tests.rs - replaced by modern JSON-driven test_runner
- d0d0193 Created 2 variants for ast_pipeline, with and without _bootstrap. Amended Cargo.toml accordingly
- f30d0ce feat: Complete AST-based parser generation with variable scoping fixes and test_runner integration
- 2eb0aa2 Enhanced parser logging: filename-specific logs and branch-level debugging
- 437f019 fix: AST-based parser generation now produces properly formatted Rust code
- 78a5959 feat: AST-based parser generation with professional debug logging
- d4e3a01 feat: AST-based code generation logging - professional formatting
- dcd3971 feat: AST-based code generation logging - inline file information
- eb892b1 feat: AST-based code generation logging - refined formatting
- 8b82eda feat: AST-based code generation logging - improved readability
- 168a6bf feat: AST-based code generation logging - perfect spacing
- 98f7707 Slightly refactored a few debug messages
- 5e96bfb Stabilize bootstrap/generated parser regressions and align parser-target contracts
- ae9ad7e Add AST-based stimuli generation mode with weighted branch probabilities
- 6e8a6a7 Harden generated parser consumption with AST rewrite and full-parse APIs
- 50c86cd Harden generated parser matching and parseability reporting
- f602a77 Add regression gate suites for whitespace and dotted identifiers
- 53ef8b3 Expand semantic edge regressions and harden regex stimuli sampling
- 573affd Add coverage-guided branch steering to stimuli generator OR selection
- 1222c3f fix target-drive semantic stall by adding stagnation probes and safer branch/quantifier guidance
- 217b2ff Align builtin bootstrap annotation conformance with parser behavior
- 589c277 Kick off SOTA roadmap with fixed-point bootstrap reproducibility gate
- 526b6d5 Wire fixed-point gate into CI and start Phase B annotation validation
- 45ec2b7 Enforce strict annotation validation in CI gate path
- ff9bf13 Add fixed-point CI artifact retention and Phase C fuzz replay mode
- 331c595 Complete Phase C: shrinking + gap-priority stimuli mode
- 1e390a6 Complete Phase D gates: differential harness, performance budgets, embedding API contract
- 35d00a9 Add differential baseline regression gate and track user-guide backlog
- 40d2671 Wire differential regression gate into CI
- 041fc10 Publish comprehensive PGEN User Guide and complete Phase E docs task
- f20b8d3 Start Pillar 2 normative annotation contractization.
- 126378c Wire annotation contract gate into CI.
- 00ee644 Extend annotation contracts with shared bootstrap/generated conformance.
- d130a47 Harden embedding API with bounded input limits.
- 36b5e4f Enforce semantic annotation leverage contract
- e453f2e Align semantic transform parsing across validator/codegen/stimuli
- ceecc19 Add EBNF frontend readiness gate for Rust migration
- a2d8565 Add annotation robustness gate to harden advanced return/semantic flows
- 582bf82 Add SOTA aggregate exit gate and sync differential baselines
- 9a43ff4 Enforce policy-driven SOTA release gate criteria
- a26eb10 Add semantic steering control matrix and return no-compromise contract
- 3c7886f Capture built-in vs annotation balance and Phase J priorities
- bee66ad Phase J: typed semantic directive registry and precedence/associativity steering baseline
- 191a62b Phase J P0: value-domain semantic steering baseline and validator payload diagnostics
- 9a13167 Phase J P1: deterministic semantic conflict-resolution baseline
- f447a88 Phase J P1: add unsatisfiable semantic value-domain diagnostics
- 2c66503 Phase J P1: add return parity gate for comparable differential corpus
- bc9c4b6 Phase J return differential burn-down: baseline 9 -> 7
- 74c2d53 Phase J return differential burn-down: baseline 7 -> 2
- 4e1c07b Close return differential debt to zero and update roadmap/docs
- 131eaa2 Fix ebnf.spec include parsing for ebnf.ebnf and promote strict EBNF gate
- 5358871 Add generated ebnf.json artifact for ebnf.ebnf frontend flow
- 3667805 Wire raw AST annotation handling end-to-end in non-bootstrap pipeline
- 6e7a3ef Add non-bootstrap annotation E2E gate target and script
- 8773e7f Enforce non-bootstrap annotation E2E gate in CI and aggregate SOTA policy
- 5c1cf25 Introduce generated parser registry for parseability validation
- ae90efe Fix non-bootstrap EBNF parser codegen for return transforms and unresolved rule references
- 2b953f3 Generalize frontend parser hardening and fix UTF-8/layout full-parse issues
- de8ca8b Wire EBNF dual-run differential into SOTA gates and untrack generated ebnf.json
- d399f47 Ignore generated/ entirely and untrack remaining generated artifacts
- ca655fd Kick off Pillar 6 with grammar ambiguity prefix diagnostics
- b87d23b Extend grammar ambiguity diagnostics with FIRST/nullable analysis
- fb6793d Add typed branch-policy/recovery directive contracts and Phase K completion
- db20f12 Phase K recovery runtime baseline: wire executable @recover/@sync/@panic_until hooks into generated OR parsing, add semantic usage recovery tests, and update roadmap/spec/UG/change docs.
- 9f0c464 Phase K SC-07 stimuli baseline: add @recover/@sync/@panic_until-driven OR-failure fallback marker emission in stimuli generation, add semantic usage tests, and update roadmap/spec/UG/change logs.
- 3b63228 Phase K SC-07 hardening: add structured recovery event reporting to generated parsers (typed RecoveryEvent/RecoveryMarkerKind, event accessors, and recovery hook event capture), plus semantic usage tests and roadmap/spec/UG/changelog updates.
- e3a787b Expand SC-07 User Guide coverage with a dedicated deep-dive section and examples; log the documentation increment in CHANGES.md and DEVELOPMENT_NOTES.md.
- 2e6ce06 Add SC-07 recover_budget enforcement and start SC-09 validator contracts
- ea37702 Harden Rust EBNF frontend and close dual-run compile regression
- 11961e0 Promote SC-09 to parser runtime baseline with generated relational enforcement
- 1962c28 Promote SC-09 to stimuli runtime relational synthesis baseline
- e13b9e3 Implement SC-07 scoped recovery budgets and align docs/contracts
- 49f377b Add SC-07 dedicated recovery-focused stimuli modes and CLI wiring
- c8d6bf8 Harden SC-09 stimuli nested relational path resolution
- bad69a7 SC-09: harden stimuli unsatisfiable-contract diagnostics
- f3e77f8 SC-09: add non-structured nested reference extraction in stimuli
- 1c24a87 SC-10: add typed coverage-target semantic steering baseline
- 4f80b41 Phase K SC-10: add parser runtime instrumentation hooks for coverage-target semantics. Generate CoverageTargetEvent + rule/branch counters/accessors, wire selected-branch tagging, add semantic usage tests, and update roadmap/spec/UG/changelog notes.
- 55cf0b7 Phase K strict-warning policy controls: add selective semantic warning promotion in strict mode. Wire PGEN_STRICT_SEMANTIC_WARNING_CODES (codes/all/none), set strict default SC-10 payload escalations, add validator regression tests, and update roadmap/spec/UG/changelog notes.
- 45ba1c7 Implement SC-11/SC-12 semantic steering baselines
- 0aa69f9 Promote SC-12 parser-side deterministic partition steering
- 933a410 Implement SC-04 token-family steering baseline across validator/parser/stimuli.
- 5738655 SC-12 runtime partition hardening for generated parsers
- 4881016 Promote SC-04 to Tier-4 gate-enforced contract
- 789beb1 Add non-annotation EBNF closed-loop quality gate
- 46caed2 Document gate terminology and add stimuli-module roadmap track
- d4a8595 Add SC-03 contract gate assets and annotation closure roadmap updates
- f0fcdae Promote SC-06 to Tier-4 contract gate
- 6b42b45 Promote SC-07 to Tier-4 recovery/sync contract gate
- ea3ac28 Promote SC-09 to Tier-4 relational constraint gate
- 09e9533 Promote SC-10 to Tier-4 coverage-target contract gate
- 679cbc7 Promote SC-11 to Tier-4 negative-case contract gate
- c34486a Promote SC-12 deterministic partitioning to Tier-4 contract gate.
- 4a08893 Promote SC-05 precedence/associativity steering to Tier-4 contract gate.
- c735bc6 Promote SC-08 value-domain steering to Tier-4 contract gate.
- e35b9f0 RA-02 runtime closure increment: identifier + single-quote return support
- 4b123e8 RA-03 increment: generated return round-trip now canonical typed output
- e8a1a1e RA-04 increment: wire explicit return full-contract gates
- 4e41091 Phase L RA-01 increment: generated return typed-AST entry path hardening
- 03a4c6e RA-01: switch generated return conversion to structural parse-tree mapping
- 5bdaf8a RA-01: use generated parse-tree conversion in generated return round-trip
- f73ca9d SA-01: use generated semantic parse-tree conversion in generated round-trip
- 9f83f7c Phase L gate closure: annotation_100 aggregate + determinism hardening
- a51ba5a Promote builtin return parseability in non-annotation quality contract.
- dd473cb Start Phase N with Rust stimuli-module generation mode.
- 08e853d phase-n: lock deterministic stimuli-module contract
- 3e7c985 phase-n: add stimuli-module parity gate and policy wiring
- d7c26c7 docs: close final phase-n stimuli-module documentation item
- 7b81048 phase-m: promote builtin semantic parseability to required
- 1ea3825 semantic parity gate alignment + non-bootstrap semantic AST hardening
- d03da73 Close return typed-AST corpus proof and advance semantic corpus conversion contracts
- 1dd68e3 Harden non-bootstrap named semantic handling with strict validated path and safe compatibility fallback
- cdd336c Close Phase L semantic typed-AST closure item after aggregate typed-AST gate validation
- 8cdef2a Promote EBNF parseability to required in non-annotation quality gate
- 854d115 Promote EBNF dual-run strict mode to required SOTA aggregate policy
- f140b2a Add tracked MEMORY.md for live session continuity and recovery handoff
- 10b719a feat: add unified tracing with trace.log file routing
- ac30884 feat: enforce file/function/line on every trace message
<!-- SESSION_GIT_HISTORY_END -->

## Binding Workflow Rules (Do Not Break)
- After each completed task, run commit workflow automatically.
- Commit workflow is:
  1. amend `git_message_brief.txt` with concise summary
  2. stage intended tracked files only
  3. `git commit -F git_message_brief.txt`
  4. clear `git_message_brief.txt` to 0 bytes
  5. keep `git_message_brief.txt` untracked
- After each completed task, update this file:
  - current snapshot values,
  - recent work summary,
  - session git history block (`origin/main..HEAD` hash/message list).
- `questions_keep_untracked.txt` must remain untracked.
- Generated artifacts under `generated/` are not authoritative state and may be overwritten/regenerated.
- `--bootstrap-mode` is reserved for generating:
  - `generated/return_annotation_parser.rs`
  - `generated/semantic_annotation_parser.rs`
- For other grammars (`json`, `regex`, `ebnf`, generic `foolang`), use non-bootstrap path.

## Recent Work Summaries (Root Cause -> Fix -> Validation)

### 2026-02-27: Phase P semantic-closure increment - dedicated `sv_semantic_file` mode
- Root cause:
  - semantic validators were wired but strict semantic-closure execution lacked a dedicated contract mode and activation switch, making focused semantic hardening clumsy.
- Fix:
  - updated `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` (`v11 -> v12`) with new mode:
    - `sv_semantic_file`
    - enabled semantic overrides:
      - `require_port_binding_legality_basic`
      - `require_package_qualification_resolution`
      - `require_context_legality_basic`
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=0|1`,
    - auto-selects `sv_semantic_file` when semantic-closure mode is enabled and explicit mode override is not provided.
  - synced roadmap + UG semantic-closure/mode docs.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - baseline mode remains stable; semantic-closure-focused execution path is now contractized and directly runnable.

### 2026-02-27: Phase P closure - dedicated Nexsim parser embedding contract gate
- Root cause:
  - Nexsim parser-profile embedding APIs existed, but there was no dedicated executable gate proving parser-profile contract behavior and no explicit metadata publication for zero-copy/session invariants.
- Fix:
  - added `rust/scripts/nexsim_parser_embedding_contract_gate.sh` and `make nexsim_parser_embedding_contract_gate`,
  - wired this target into `embedding_api_gate`,
  - hardened `ParserEmbeddingApiContract` (`rust/src/embedding_api.rs`) with:
    - `input_ownership_model=borrowed_str`,
    - `parse_session_model=stateless_per_call`,
    - `zero_copy_input_boundary=true`,
    - stable parser diagnostic code publication,
  - added convenience-wrapper parity tests for:
    - `parse_systemverilog_2017/2023`,
    - `parse_vhdl_1076_2019`.
  - marked roadmap Nexsim parser embedding profile-contract item complete and synced docs (`PGEN_USER_GUIDE.md`, `rust/docs/EMBEDDING_API_CONTRACT.md`).
- Validation:
  - `make -C rust SHELL=/opt/homebrew/bin/bash nexsim_parser_embedding_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash embedding_api_gate`
- Status:
  - parser-profile embedding contract is now explicitly published and continuously gate-checked in both bootstrap and generated modes.

### 2026-02-27: Aggregate policy promotion - `sv_stimuli_quality_gate` required by default
- Root cause:
  - SV stimuli gate remained informational in aggregate policy despite closure progress on stage-order and deterministic replay invariants.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_REQUIRE_SV_STIMULI_QUALITY_STRICT=1`
  - synced roadmap + UG docs to reflect strict default policy posture.
- Validation:
  - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract PGEN_SOTA_RUN_EBNF_READINESS=0 PGEN_SOTA_RUN_ANNOTATION_ROBUSTNESS=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=0 PGEN_SOTA_RUN_STIMULI_MODULE_PARITY=0 PGEN_SOTA_RUN_HDL_FRONTEND_READINESS=0 PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0 PGEN_SOTA_RUN_SV_STIMULI_QUALITY=1 PGEN_SOTA_REQUIRE_SV_STIMULI_QUALITY_STRICT=1 PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY=0 PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sota_exit_gate`
- Status:
  - aggregate gate now requires SV stimuli quality pass under tracked defaults.

### 2026-02-27: Phase P deterministic initial replay checks in SV closed-loop gate
- Root cause:
  - closed-loop convergence used deterministic seeds but did not explicitly assert initial-stage replay equivalence as a gate condition.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added `profile_<lrm>_closed_loop_initial_replay` stage per profile,
    - added deterministic equivalence assertions for initial vs initial-replay artifacts:
      - stimuli text,
      - coverage JSON (canonical compare),
      - gap JSON (canonical compare),
      - gap text.
    - added summary metric:
      - `closed_loop_initial_replay_determinism_passes`.
  - updated roadmap/UG/docs continuity and marked SV closed-loop convergence item complete.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - deterministic initial replay equivalence is now an executable gate invariant for SV profiles.

### 2026-02-27: Phase Q/P stage-order alignment in `sv_stimuli_quality_gate`
- Root cause:
  - Phase Q parser/stimuli contract requires `preprocess -> parse_full -> semantic-validate`, but gate sample loop executed semantic validation before parse-full.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - reordered per-sample stage flow to:
      - `stimuli_generate -> preprocess -> parse_full(optional) -> semantic_validate_baseline`
    - retained strict parse-full immediate-fail semantics,
    - preserved parse stage status in semantic-failure summary rows.
  - updated roadmap + UG:
    - parser/stimuli integration contract item marked complete,
    - sample-stage order text aligned with implementation.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - sample-stage execution order now matches Phase Q parser/stimuli integration contract.

### 2026-02-27: Phase Q/P SV stimuli gate now tracks preprocess convergence debt in closed loop
- Root cause:
  - `sv_stimuli_quality_gate` enforced parser target-debt convergence but did not track preprocessor diagnostics debt across closed-loop initial/replay stages.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added per-profile closed-loop preprocess passes for initial and replay corpora,
    - added summary metrics:
      - `closed_loop_initial_preprocess_warnings_total`
      - `closed_loop_initial_preprocess_errors_total`
      - `closed_loop_replay_preprocess_warnings_total`
      - `closed_loop_replay_preprocess_errors_total`
    - extended `closed_loop.require_non_increasing_target_debt` enforcement to include preprocess error debt non-increase (`replay_preprocess_errors <= initial_preprocess_errors`) per profile.
  - synced roadmap + UG descriptions for dual parser+preprocess convergence semantics.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - SV closed-loop convergence now includes preprocess diagnostics debt alongside parser gap debt.

### 2026-02-27: Aggregate policy promotion - `sv_preprocessor_quality_gate` required by default
- Root cause:
  - preprocessor gate was integrated in aggregate flow but still informational, leaving Phase Q enforcement weaker than intended.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT=1`
  - synced roadmap + UG docs with new default policy posture.
- Validation:
  - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract PGEN_SOTA_RUN_EBNF_READINESS=0 PGEN_SOTA_RUN_ANNOTATION_ROBUSTNESS=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN=0 PGEN_SOTA_RUN_STIMULI_MODULE_PARITY=0 PGEN_SOTA_RUN_HDL_FRONTEND_READINESS=0 PGEN_SOTA_RUN_SV_STIMULI_QUALITY=0 PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY=0 PGEN_SV_PREPROCESSOR_QUALITY_COUNT=1 PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS=1 make -C rust SHELL=/opt/homebrew/bin/bash sota_exit_gate`
- Status:
  - aggregate gate now requires SV preprocessor quality pass under tracked defaults.

### 2026-02-27: Phase Q trusted-reference differential taxonomy in SV preprocessor gate
- Root cause:
  - `sv_preprocessor_quality_gate` had deterministic closed-loop/fuzz checks but no trusted-reference differential layer or published mismatch taxonomy.
- Fix:
  - updated `rust/scripts/sv_preprocessor_quality_gate.sh`:
    - added differential controls:
      - `PGEN_SV_PREPROCESSOR_DIFF_MODE=auto|0|1`
      - `PGEN_SV_PREPROCESSOR_DIFF_MAX_SAMPLES`
      - `PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER`
    - added trusted-reference runner execution stage on generated samples,
    - added deterministic taxonomy classification and per-sample artifact recording,
    - added report output:
      - `rust/target/sv_preprocessor_quality_gate/work/systemverilog_preprocessor_differential_report.json`
    - strict mode now fails on runner absence or any non-`match` taxonomy.
  - updated roadmap + UG with taxonomy definitions and runner interface contract.
- Validation:
  - `bash -n rust/scripts/sv_preprocessor_quality_gate.sh`
  - `PGEN_SV_PREPROCESSOR_QUALITY_COUNT=1 PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS=1 make -C rust SHELL=/opt/homebrew/bin/bash sv_preprocessor_quality_gate`
  - `PGEN_SV_PREPROCESSOR_QUALITY_COUNT=1 PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS=1 PGEN_SV_PREPROCESSOR_DIFF_MODE=1 PGEN_SV_PREPROCESSOR_DIFF_MAX_SAMPLES=1 PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER=/tmp/pgen_svpp_reference_runner.sh make -C rust SHELL=/opt/homebrew/bin/bash sv_preprocessor_quality_gate`
- Status:
  - taxonomy mechanics are now executable; external trusted-reference adapters can be plugged in without script changes.

### 2026-02-27: Phase Q preprocess-aware SV stimuli mode expansion (`sv_pp_file`, `sv_pp_snippet`)
- Root cause:
  - Phase Q parser/stimuli integration roadmap required explicit preprocess-aware stimuli modes, but contract/gate mode set only exposed `sv_file`/`sv_snippet`.
- Fix:
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` from `v10 -> v11`.
  - added supported modes + profiles:
    - `sv_pp_file` (file-oriented, closed-loop enabled, parse-full eligible, recovery `recovery_biased`),
    - `sv_pp_snippet` (snippet-oriented, closed-loop disabled by default, parse-full ineligible, recovery `near_sync_negative`).
  - hardened fallback mode mapping in `rust/scripts/sv_stimuli_quality_gate.sh` so preprocess-aware mode names are recognized for entry-rule, closed-loop, and parse-full defaults.
  - synced roadmap + UG docs for the new mode set.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_pp_file PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_pp_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - preprocess-aware mode plumbing is now executable and contractized for Phase Q integration work.

### 2026-02-27: Aggregate SOTA wiring for dedicated VHDL quality gate
- Root cause:
  - `vhdl_stimuli_quality_gate` existed but aggregate `sota_exit_gate` did not execute it, so VHDL closed-loop hardening was not part of default aggregate quality flow.
- Fix:
  - updated `rust/config/sota_exit_policy.env` with default VHDL aggregate toggles:
    - `PGEN_SOTA_POLICY_RUN_VHDL_STIMULI_QUALITY=1`
    - `PGEN_SOTA_POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT=0`
  - updated `rust/scripts/sota_exit_gate.sh` to:
    - ingest policy/runtime VHDL toggles,
    - validate both toggles as `0|1`,
    - print effective VHDL mode in summary,
    - execute `vhdl_stimuli_quality_gate` as informational or required based on strictness.
  - updated roadmap/UG/changelog/dev-notes continuity docs.
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh`
  - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract PGEN_SOTA_RUN_EBNF_READINESS=0 PGEN_SOTA_REQUIRE_EBNF_STRICT=0 PGEN_SOTA_RUN_ANNOTATION_ROBUSTNESS=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN=0 PGEN_SOTA_RUN_STIMULI_MODULE_PARITY=0 PGEN_SOTA_RUN_HDL_FRONTEND_READINESS=0 PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0 PGEN_SOTA_RUN_SV_STIMULI_QUALITY=0 PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY=1 PGEN_SOTA_REQUIRE_VHDL_STIMULI_QUALITY_STRICT=0 PGEN_VHDL_STIMULI_QUALITY_COUNT=1 PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sota_exit_gate`
- Status:
  - aggregate SOTA now executes VHDL closed-loop quality checks by policy default (informational-first), with strict promotion switch available.

### 2026-02-27: Phase O Nexsim VHDL closure increment - dedicated `vhdl_stimuli_quality_gate`
- Root cause:
  - VHDL had aggregate HDL readiness coverage but no dedicated contractized closed-loop quality gate equivalent to SV flow hardening.
- Fix:
  - added `rust/scripts/vhdl_stimuli_quality_gate.sh`:
    - deterministic `EBNF -> JSON -> parser -> coverage/gap(initial) -> replay -> parse_full(optional)` gate flow,
    - dynamic parseability adapter build via `PGEN_VHDL_PARSER_PATH`,
    - parse-full modes `auto|0|1`,
    - contractized non-increasing closed-loop target debt check.
  - added contract manifest `rust/test_data/grammar_quality/vhdl_core_v0_contract.json` (v1).
  - added make target `make -C rust vhdl_stimuli_quality_gate` and help wiring.
  - updated roadmap + user guide with gate semantics and tuning variables.
- Validation:
  - `bash -n rust/scripts/vhdl_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/vhdl_core_v0_contract.json`
  - `PGEN_VHDL_STIMULI_QUALITY_COUNT=1 PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash vhdl_stimuli_quality_gate`
  - `PGEN_VHDL_STIMULI_QUALITY_COUNT=1 PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/opt/homebrew/bin/bash vhdl_stimuli_quality_gate`
- Status:
  - dedicated VHDL closed-loop stimuli quality gating is now executable and deterministic for Nexsim-focused hardening.

### 2026-02-27: Phase P stimuli-mode recovery steering (contract v10)
- Root cause:
  - stimuli modes were present, but per-mode recovery strategy steering was not contractized/routed to generator invocations.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added `stimuli_modes.profiles.<mode>.recovery_stimuli_mode` resolution + validation,
    - forwarded `--recovery-stimuli-mode` on closed-loop initial/replay and per-sample generation calls,
    - exposed effective `stimuli_mode_recovery_stimuli_mode` in gate summary header.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v10`:
    - `sv_file.recovery_stimuli_mode = baseline`
    - `sv_snippet.recovery_stimuli_mode = near_sync_negative`
  - updated roadmap + UG documentation for mode-level recovery steering.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - mode-level stimuli steering now includes deterministic recovery strategy selection; further annotation-driven branch/value steering expansion remains pending.

### 2026-02-27: Phase P stimuli-mode semantic overrides (contract v9)
- Root cause:
  - stimuli modes existed, but semantic baseline strictness was global only and could not be tuned per mode.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh` to apply mode-level semantic overrides:
    - `stimuli_modes.profiles.<mode>.semantic_overrides.<semantic_baseline_toggle>`
    - effective semantic checks now resolve as global defaults overridden by selected mode profile.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v9`:
    - `sv_file` enables `require_port_binding_legality_basic`
    - `sv_snippet` disables `require_port_binding_legality_basic`
  - updated roadmap + UG docs for mode semantic override behavior.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - mode-level semantic strictness bridge is now in place; semantic-annotation-driven branch/value generation steering remains pending.

### 2026-02-27: Phase P context legality extension - generate-loop `genvar` baseline
- Root cause:
  - context-legality coverage lacked executable generate-loop legality checks.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh` `check_context_legality_basic`:
    - parse declared `genvar` names,
    - inspect `generate ... endgenerate` `for` iterators,
    - fail when iterator is not declared `genvar`.
  - reused existing `semantic_baseline.require_context_legality_basic` toggle (no schema change).
  - updated roadmap + UG to reflect expanded context-legality coverage.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - context-legality baseline now covers always/context + generate-loop `genvar` legality.

### 2026-02-27: Phase P semantic closure - basic port-binding legality toggle (contract v8)
- Root cause:
  - semantic closure profile still missed basic legality checking for named port bindings in known in-file module instantiations.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added `check_port_binding_legality_basic` validator,
    - wired toggle into `evaluate_semantic_baseline`.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v8`:
    - added `semantic_baseline.require_port_binding_legality_basic` (default `false`).
  - updated roadmap + UG semantic-closure documentation.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - semantic closure now includes a deterministic basic named-port legality control; broader semantic strictness rollout remains pending.

### 2026-02-27: Phase P deterministic failure replay/shrinking (SV stimuli gate, contract v7)
- Root cause:
  - closed-loop convergence item needed deterministic failing-sample replay with minimized artifacts; gate reported failures but did not shrink failing cases.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh`:
    - added unified semantic evaluator (`evaluate_semantic_baseline`) used by normal stage + replay predicates,
    - added contract-driven deterministic prefix shrinker (`deterministic_prefix_shrink`) and failure predicates for semantic/parse-full failures,
    - added failure-replay summary counters (`semantic_failures_shrunk`, `parse_full_failures_shrunk`),
    - added CSV note sanitization to keep summary rows stable.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v7` and added:
    - `failure_replay.enabled`
    - `failure_replay.shrink_semantic_failures`
    - `failure_replay.shrink_parse_full_failures`
    - `failure_replay.shrink_max_iterations`
  - updated roadmap + UG to document failure replay/shrinking contract.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - deterministic failure replay/shrinking plumbing is in place; semantic-annotation-driven SV steering remains pending.

### 2026-02-27: Phase P stimuli mode plumbing (`sv_file` / `sv_snippet`)
- Root cause:
  - SV stimuli gate lacked explicit mode selection for full-file vs snippet generation; entry-rule behavior was implicit and not contractized.
- Fix:
  - added contractized stimuli modes in `systemverilog_core_v0_contract.json` (v6):
    - `stimuli_modes.default_mode`
    - `stimuli_modes.supported_modes`
    - per-mode profile fields:
      - `entry_rule`
      - `closed_loop_enabled`
      - `parse_full_eligible`
  - updated `sv_stimuli_quality_gate.sh` to:
    - resolve mode from contract/env (`PGEN_SV_STIMULI_QUALITY_MODE`),
    - pass mode-specific `--entry-rule` on all generation paths,
    - gate closed-loop by mode eligibility,
    - gate parse-full strict mode by mode eligibility.
  - updated roadmap + UG for mode semantics and usage.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_snippet PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - explicit `sv_file`/`sv_snippet` generation mode infrastructure is now in place; semantic-annotation-driven steering remains a follow-up.

### 2026-02-27: Phase P semantic-closure validator wiring (SV stimuli gate)
- Root cause:
  - semantic-closure profile item needed executable checks for declaration/use, package qualification resolution, width compatibility, and basic always-context legality; existing gate only had structural/preprocess checks.
- Fix:
  - expanded `rust/scripts/sv_stimuli_quality_gate.sh` with new optional validators:
    - `check_declared_identifiers_before_use`
    - `check_package_qualification_resolution`
    - `check_width_compatibility_simple`
    - `check_context_legality_basic`
  - contractized toggles in `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` (version `5`):
    - `require_declared_identifiers_before_use`
    - `require_package_qualification_resolution`
    - `require_width_compatibility_simple`
    - `require_context_legality_basic`
    (defaulted `false` to keep current corpus baseline stable while wiring lands.)
  - updated roadmap + UG to reflect semantic-closure validator availability.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - semantic-closure validator infrastructure is wired and contract-controlled; stricter enablement can now be phased profile-by-profile.

### 2026-02-27: Phase P syntax-closure burn-down loop implementation
- Root cause:
  - Phase P still lacked an executable deterministic no-regression gate specifically for SystemVerilog syntax closure; closure state was mostly tracked by docs/manual scans.
- Fix:
  - added `rust/scripts/sv_syntax_closure_gate.sh` and contract `rust/test_data/grammar_quality/systemverilog_syntax_closure_contract.json`.
  - gate enforces:
    - parser generation viability (`EBNF -> JSON -> parser`),
    - unresolved reference budget,
    - unique rule names + entry-rule presence,
    - reachable/unreachable rule+branch caps from deterministic gap summary.
  - added Make target: `make -C rust sv_syntax_closure_gate`.
  - updated roadmap/UG to mark syntax burn-down loop as implemented and documented.
- Validation:
  - `bash -n rust/scripts/sv_syntax_closure_gate.sh`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_syntax_closure_gate`
- Status:
  - Phase P syntax closure now has an executable, contractized burn-down gate.

### 2026-02-27: Phase P `systemverilog_core_v0` freeze and `sv_stimuli_quality_gate` closed-loop promotion
- Root cause:
  - SV quality gate still operated as a preprocess/semantic/parse skeleton and did not enforce profile-level `coverage/gap -> target replay` closure required by Phase P contract intent.
- Fix:
  - promoted `rust/scripts/sv_stimuli_quality_gate.sh` to run deterministic per-profile closed-loop stages:
    - initial `coverage + gap` extraction
    - target-driven replay with `--target-report-input`
    - contract-enforced non-increasing target-debt check.
  - expanded summary schema with closed-loop stage columns:
    - `coverage_gap_initial`, `gap_replay`.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `version: 4` and added `closed_loop` controls:
    - `enabled`, `gap_report_threshold`, `target_max_attempts`, `replay_sample_count`, `require_non_increasing_target_debt`.
  - aligned `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, `PGEN_USER_GUIDE.md`, and `rust/Makefile` help text.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - Phase P roadmap item `Freeze systemverilog_core_v0 contract corpus and add sv_stimuli_quality_gate` now implemented as closed-loop baseline.

### 2026-02-27: Nexsim SV/VHDL focus increment (convenience APIs + HDL parseability stage)
- Root cause:
  - Nexsim integration needed lower-friction SV/VHDL parser call sites and explicit visibility into whether generated HDL parsers can replay emitted stimuli end-to-end.
- Fix:
  - added SV/VHDL convenience embedding API entry points in `rust/src/embedding_api.rs`:
    - `parse_systemverilog_2017*`, `parse_systemverilog_2023*`, `parse_vhdl_1076_2019*`
    - parser embedding contract now includes per-grammar `profile_matrix`.
  - extended `rust/scripts/hdl_frontend_readiness_gate.sh` with parser replay stages:
    - `parser_registry_support`
    - `parseability`
    - probe build + adapter support + per-sample replay now reported in summary.
    - added deterministic retry-to-parseable loop (controlled by `PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS`) using per-sample files/manifests to avoid multiline-sample line-splitting failures.
  - resolved generated-parser compatibility drift used by probe builds:
    - `rust/src/lib.rs`: semantic parser backward-compat alias (`Semantic_annotationParser` -> `SemanticAnnotationParser`).
    - `rust/src/ast_pipeline/ast_based_generator.rs`: replaced stale `parser.debug_output` codegen with logger-based debug emission.
    - regenerated annotation parsers (`make -C rust return_annotation_parser semantic_annotation_parser`) to absorb the codegen fix.
- Validation:
  - `PGEN_HDL_FRONTEND_STIMULI_COUNT=1 PGEN_HDL_FRONTEND_STRICT=0 make -C rust SHELL=/opt/homebrew/bin/bash hdl_frontend_readiness`
  - `PGEN_HDL_FRONTEND_STIMULI_COUNT=3 PGEN_HDL_FRONTEND_STRICT=1 make -C rust SHELL=/opt/homebrew/bin/bash hdl_frontend_gate`
  - `cargo test --manifest-path rust/Cargo.toml --lib embedding_api`
- Status:
  - strict HDL gate now passes for both tracked grammars (`systemverilog`, `vhdl`) including parser-registry parseability stage.

### 2026-02-27: Embedding API convention hardening (Rust + non-Rust)
- Root cause:
  - parser embedding API needed lower-friction host ergonomics beyond typed-outcome-only calls to align with common Rust and cross-language integration patterns.
- Fix:
  - added idiomatic Rust `Result` wrappers (`*_result` APIs) in `rust/src/embedding_api.rs`.
  - added named string-based parse entry points (`*_named` APIs) for binding/FFI layers.
  - added canonical string mapping (`as_str`) and `FromStr` aliases for family/backend/grammar/profile types.
  - added `Display` + `std::error::Error` implementations for `ParseDiagnostic`.
  - added deterministic invalid-argument diagnostic for named APIs: `E_INVALID_ARGUMENT`.
  - aligned contract/UG/roadmap docs with the new dual-surface API shape.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml --lib embedding_api`
  - `cargo test --manifest-path rust/Cargo.toml --features generated_parsers --lib embedding_api`

### 2026-02-27: Nexsim parser embedding API profile scaffold (SV/VHDL)
- Root cause:
  - embedding API exposed annotation parsing only; Nexsim-targeted Phase P requires stable grammar parser entry points with explicit profile contracts.
- Fix:
  - added build-time optional backend discovery for generated SV/VHDL parser artifacts in `rust/build.rs`.
  - extended generated parser module exports and parser registry with optional VHDL adapter path.
  - added profile-aware parser embedding API in `rust/src/embedding_api.rs`:
    - `parser_embedding_api_contract()`
    - `parse_grammar_profile(...)`
    - `parse_grammar_profile_with_limits(...)`
    - stable enums for grammar/profile matrix (`systemverilog`/`vhdl`, `sv_2017`/`sv_2023`/`vhdl_1076_2019`)
    - deterministic diagnostics for profile mismatch/unavailable backends.
  - aligned docs and roadmap entries with the new contract surface.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml --lib embedding_api`.

### 2026-02-27: Common `systemverilog.ebnf` dual-LRM scaffold (`2017|2023`)
- Root cause:
  - agreed direction is one common SystemVerilog grammar for both LRMs; gate/contract path lacked explicit profile controls.
- Fix:
  - extended `rust/scripts/sv_stimuli_quality_gate.sh` with profile controls:
    - `PGEN_SV_STIMULI_QUALITY_LRM_PROFILE`
    - `PGEN_SV_STIMULI_QUALITY_LRM_PROFILES`
  - extended `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to version `3` with:
    - `lrm_profiles.default_profile`
    - `lrm_profiles.supported_profiles`
    - `lrm_profiles.required_profiles`
  - gate now runs profile matrix (`2017`, `2023`) and emits profile-tagged summary rows.
  - roadmap now includes explicit Nexsim parser embedding API profile contract task.
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` (both profiles executed).

### 2026-02-27: Added dual-LRM conversion tooling/workspaces (1800-2023 + 1076-2019)
- Root cause:
  - older script flow was tied to one specific 2017 extraction setup and not reusable for newer LRMs without manual rework.
- Fix:
  - added adapted, reusable tooling under `tools/`:
    - `split_sections.py`, `txt_to_md_converter.py`, `extract_grammar.py`, `extract_grammar_v2.py`, `create_clean_grammar.py`, `ieee_lrm_converter.py`
  - added workflow doc:
    - `tools/LRM_CONVERSION_WORKFLOW.md`
  - created local workspaces:
    - `docs/systemverilog/`
    - `docs/vhdl/`
  - added workspace `.gitignore` rules so generated conversion outputs remain untracked.
  - fixed clause matcher in `split_sections.py` to accept TOC headings like `1. Overview`.
- Validation:
  - `python3 tools/ieee_lrm_converter.py --pdf /Users/richarddje/Documents/github/1800-2023.pdf --out-root docs/systemverilog --document "SystemVerilog Language Reference Manual" --standard "IEEE 1800-2023" --domain "SystemVerilog" --clause-depth 1 --limit 2 --extract-grammar`
  - `python3 tools/ieee_lrm_converter.py --pdf /Users/richarddje/Documents/github/ieee-1076-2019.pdf --out-root docs/vhdl --document "VHDL Language Reference Manual" --standard "IEEE 1076-2019" --domain "VHDL" --clause-depth 1 --limit 2 --extract-grammar`

### 2026-02-27: Expanded `sv_stimuli_quality_gate` semantic baseline (contract v2)
- Root cause:
  - semantic validation stage in SV stimuli gate only checked preprocess artifacts and needed stronger semantic-contract signal.
- Fix:
  - updated `rust/scripts/sv_stimuli_quality_gate.sh` with contract-driven semantic checks:
    - active baseline: unique named-port binding detection per statement,
    - optional strict check: structural keyword-pair balancing.
  - bumped `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to version `2` and added semantic baseline toggles:
    - `require_unique_named_port_bindings` (enabled),
    - `require_balanced_structural_keywords` (currently disabled by default).
- Validation:
  - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passes with contract v2.

### 2026-02-27: Promoted aggregate HDL readiness to required strict
- Root cause:
  - after strict HDL gate turned green for both tracked grammars, aggregate policy was still informational for HDL readiness.
- Fix:
  - set `PGEN_SOTA_POLICY_REQUIRE_HDL_FRONTEND_STRICT=1` in `rust/config/sota_exit_policy.env`.
  - updated roadmap/user-guide/doc logs to reflect strict promotion.
- Validation:
  - scoped aggregate run with required baseline + HDL strict path:
    - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract`
    - `PGEN_SOTA_RUN_EBNF_READINESS=0`
    - `PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=0`
    - `PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0`
    - `PGEN_SOTA_RUN_SV_STIMULI_QUALITY=0`
    - `make -C rust SHELL=/bin/bash sota_exit_gate`

### 2026-02-27: Added `vhdl.ebnf` seed grammar and turned strict HDL readiness green
- Root cause:
  - strict HDL readiness could not pass while `grammars/vhdl.ebnf` was missing.
- Fix:
  - added executable `grammars/vhdl.ebnf` seed grammar with design-unit/declaration/statement/expression/token baseline coverage.
  - updated roadmap and user guide status to reflect strict HDL pass state.
- Validation:
  - unresolved-reference scan for `grammars/vhdl.ebnf` is empty.
  - `make -C rust SHELL=/bin/bash hdl_frontend_gate` passes for both `systemverilog` and `vhdl`.

### 2026-02-27: Wired HDL readiness into aggregate SOTA policy (informational-first)
- Root cause:
  - Phase O aggregate-policy decision was still open; HDL readiness existed as standalone commands but not in aggregate gate output.
- Fix:
  - updated `rust/scripts/sota_exit_gate.sh` with policy/runtime controls:
    - `PGEN_SOTA_POLICY_RUN_HDL_FRONTEND_READINESS`
    - `PGEN_SOTA_POLICY_REQUIRE_HDL_FRONTEND_STRICT`
    - `PGEN_SOTA_RUN_HDL_FRONTEND_READINESS`
    - `PGEN_SOTA_REQUIRE_HDL_FRONTEND_STRICT`
  - added boolean validation, summary fields, and informational/strict run branches for `hdl_frontend_readiness`/`hdl_frontend_gate`.
  - set defaults in `rust/config/sota_exit_policy.env` to informational-first (`run=1`, `strict=0`).
- Validation:
  - scoped aggregate run with HDL readiness enabled and other optional checks disabled:
    - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract`
    - `PGEN_SOTA_RUN_EBNF_READINESS=0`
    - `PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=0`
    - `PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0`
    - `PGEN_SOTA_RUN_SV_STIMULI_QUALITY=0`
    - `PGEN_SOTA_RUN_HDL_FRONTEND_READINESS=1`
    - `make -C rust SHELL=/bin/bash sota_exit_gate`

### 2026-02-27: Closed unresolved symbol references in `systemverilog.ebnf`
- Root cause:
  - matrix-driven unresolved-reference scan showed five missing symbols blocking clean syntax-consistency tracking.
- Fix:
  - updated `grammars/systemverilog.ebnf` by adding:
    - `modport_declaration`, `modport_item`, `modport_ports_declaration`, `modport_port`
    - `checker_instantiation`
    - `block_item_declaration`
    - `class_item`
    - `kw_assert`
  - refreshed:
    - `SV_GRAMMAR_COVERAGE_MATRIX.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
    - `PGEN_USER_GUIDE.md`
- Validation:
  - unresolved-reference scan now returns empty.
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`

### 2026-02-27: Added Annex-A anchored SystemVerilog grammar coverage matrix
- Root cause:
  - Phase P required concrete syntax-closure tracking tied to IEEE anchors and explicit per-rule status, but no dedicated matrix artifact existed.
- Fix:
  - added `SV_GRAMMAR_COVERAGE_MATRIX.md` with:
    - Annex-A-aligned seed coverage table,
    - grouped per-rule inventory from `grammars/systemverilog.ebnf`,
    - unresolved-reference debt list (`block_item_declaration`, `checker_instantiation`, `class_item`, `kw_assert`, `modport_declaration`),
    - refresh procedure for keeping matrix executable-evidence aligned.
  - updated roadmap and UG pointers:
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
    - `PGEN_USER_GUIDE.md`
- Validation:
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness`
  - unresolved-reference scan result matches matrix debt list.

### 2026-02-27: Wired `sv_stimuli_quality_gate` into aggregate SOTA policy (informational-first)
- Root cause:
  - `sv_stimuli_quality_gate` was standalone and not visible in aggregate SOTA execution policy, so release-grade one-shot gate runs did not report this Phase Q/P signal.
- Fix:
  - updated `rust/scripts/sota_exit_gate.sh` with policy + runtime controls:
    - `PGEN_SOTA_POLICY_RUN_SV_STIMULI_QUALITY`
    - `PGEN_SOTA_POLICY_REQUIRE_SV_STIMULI_QUALITY_STRICT`
    - `PGEN_SOTA_RUN_SV_STIMULI_QUALITY`
    - `PGEN_SOTA_REQUIRE_SV_STIMULI_QUALITY_STRICT`
  - added boolean validation, summary printing, and `run_check` branch for `sv_stimuli_quality_gate`.
  - updated `rust/config/sota_exit_policy.env` defaults:
    - run enabled (`1`), strict disabled (`0`) for informational-first rollout.
- Validation:
  - scoped aggregate run with minimal required checks and only SV stimuli informational gate enabled:
    - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract`
    - `PGEN_SOTA_RUN_EBNF_READINESS=0`
    - `PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=0`
    - `PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0`
    - `PGEN_SOTA_RUN_SV_STIMULI_QUALITY=1`
    - `make -C rust SHELL=/bin/bash sota_exit_gate`
  - run passed and summary includes `sv_stimuli_quality_gate`.

### 2026-02-27: Wired dynamic `systemverilog` parseability adapter path for `sv_stimuli_quality_gate`
- Root cause:
  - `sv_stimuli_quality_gate` skeleton existed but parse-full stage could not execute for `systemverilog` in standard generated-parser builds because the generated SV parser artifact is gate-produced and untracked.
- Fix:
  - added `rust/build.rs` with optional parser-path contract:
    - `PGEN_SYSTEMVERILOG_PARSER_PATH` + cfg `has_generated_systemverilog_parser`.
  - added conditional generated parser module wiring in `rust/src/lib.rs`.
  - added conditional `systemverilog` adapter in `rust/src/parser_registry.rs` using `parse_full_systemverilog_file`.
  - updated `rust/scripts/sv_stimuli_quality_gate.sh` to:
    - generate `systemverilog_parser.rs`,
    - build `parseability_probe` with `PGEN_SYSTEMVERILOG_PARSER_PATH=<generated parser path>`,
    - execute parse-full in auto mode with stage pass/fail accounting.
  - parse-full mode semantics now:
    - `auto`: soft-fail on parse-full rejection (record and continue),
    - `1`: strict fail on missing adapter or first parse-full rejection,
    - `0`: stage disabled.
- Validation:
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --features generated_parsers --bin ast_pipeline -q`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --features generated_parsers --bin parseability_probe -q`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo test --features generated_parsers --lib parser_registry -- --nocapture`
  - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` (auto mode: parse-full executes with pass/fail counters)
  - `PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=1 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` (expected strict failure on rejection)

### 2026-02-27: Started Phase Q/P parser-stimuli integration with `sv_stimuli_quality_gate` skeleton
- Root cause:
  - roadmap required an executable `preprocess -> parse_full -> semantic-validate` gate path for SystemVerilog stimuli, but only preprocessor-only quality gates existed.
- Fix:
  - added `rust/scripts/sv_stimuli_quality_gate.sh` and Make target `sv_stimuli_quality_gate`.
  - added initial contract manifest `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`.
  - added parser-registry probe binary `rust/src/bin/parseability_probe.rs` (+ Cargo bin wiring) so gate can explicitly test adapter availability and run parse-full checks on preprocessed samples when supported.
  - gate now executes deterministic per-sample stages:
    - stimuli generation,
    - preprocessing,
    - semantic baseline validation (`non-empty preprocessed output` + `no preprocess error diagnostics`),
    - parse-full stage (`auto|0|1` policy; currently auto-skips when adapter missing).
- Validation:
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --features generated_parsers --bin parseability_probe -q`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --features generated_parsers --bin ast_pipeline -q`
  - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
  - superseded by dynamic adapter wiring increment where parse-full is executable in auto mode.

### 2026-02-27: Closed Phase Q preprocessor semantic controls + diagnostics contract
- Root cause:
  - preprocessor execution path lacked a complete typed policy surface and structured diagnostics artifact contract, leaving Phase Q semantic-controls item open.
- Fix:
  - extended `SvPreprocessorConfig` in `rust/src/sv_preprocessor.rs` with typed policies:
    - `IncludePathPolicy`
    - `MacroRedefinitionPolicy`
    - `ConditionalSymbolPolicy`
    - `ConditionalExprPolicy`
    - `strict_warning_codes` with parser helper (`none|all|csv`).
  - added structured diagnostics model and output propagation:
    - `PreprocessorDiagnostic` (`code/severity/file/line/message/detail`)
    - `PreprocessedOutput.diagnostics`
  - hardened conditional behavior:
    - `ifdef`/`elsif` respect undefined-symbol policy and conditional-expression policy,
    - `ifndef` now uses presence-based evaluation without undefined-symbol warning inflation.
  - wired CLI policy/diagnostic flags in `rust/src/main.rs`:
    - `--sv-diagnostics-json`
    - `--sv-include-path-policy`
    - `--sv-macro-redefine-policy`
    - `--sv-conditional-symbol-policy`
    - `--sv-conditional-expr-policy`
    - `--sv-strict-warning-codes`
    - env fallback: `PGEN_SVPP_STRICT_WARNING_CODES`.
  - added targeted unit tests for macro-redefine warn/error, strict warning promotion, and `ifndef`/`ifdef` undefined-symbol policy behavior.
- Validation:
  - `cd rust && RUSTFLAGS='-Awarnings' cargo test --lib sv_preprocessor -- --nocapture`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --bin ast_pipeline --features generated_parsers -q`
  - manual CLI smoke with diagnostics JSON emission and conditional warn policy.

### 2026-02-27: Implemented Rust SV preprocessor execution stage in `ast_pipeline`
- Root cause:
  - Phase Q needed executable preprocessing behavior in Rust pipeline (not grammar/gate scaffolding only) to support true `raw SV -> expanded SV` flow and metadata handoff.
- Fix:
  - added `rust/src/sv_preprocessor.rs` with deterministic preprocessing engine:
    - `define/undef/include/ifdef-family` handling,
    - object/function macro expansion (token-paste/stringize baseline),
    - include-depth bound and include-cycle detection,
    - source-map metadata and structured event logs.
  - exported module in `rust/src/lib.rs`.
  - wired new AST-pipeline CLI mode in `rust/src/main.rs`:
    - `--preprocess-systemverilog`,
    - include/depth/redefine controls,
    - optional JSON artifacts (`--sv-source-map-json`, `--sv-event-log-json`).
  - added focused module tests for define/include/conditional/function-macro behavior.
- Validation:
  - `cd rust && RUSTFLAGS='-Awarnings' cargo test --lib sv_preprocessor -- --nocapture`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --bin ast_pipeline --features generated_parsers -q`
  - manual CLI run confirms expanded output + source-map/event-log emission.

### 2026-02-27: Added executable SV preprocessor quality gate + aggregate informational wiring
- Root cause:
  - Phase Q needed an objective, repeatable quality gate for `systemverilog_preprocessor` beyond one-shot `EBNF -> JSON -> parser -> stimuli` smoke checks.
- Fix:
  - added `rust/scripts/sv_preprocessor_quality_gate.sh` and Make target `sv_preprocessor_quality_gate` with:
    - deterministic replay checks (same-seed sample + canonical coverage/gap parity),
    - closed-loop coverage/gap progression checks (baseline -> gap-priority -> target-drive -> final-gap),
    - key preprocessor rule/branch-family coverage assertions,
    - deterministic coverage-guided fuzz replay parity checks.
  - added adaptive parseability mode (`auto|0|1`) so strict parseability/shrink behavior activates when parser-registry adapter support exists.
  - wired into aggregate policy as informational-first via:
    - `rust/scripts/sota_exit_gate.sh`
    - `rust/config/sota_exit_policy.env` (`run=1`, `strict=0`).
- Validation:
  - `make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
  - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract PGEN_SOTA_RUN_EBNF_READINESS=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=0 make -C rust SHELL=/bin/bash sota_exit_gate`
  - both passed; aggregate run confirms `sv_preprocessor_quality_gate` wiring as informational.

### 2026-02-27: Implemented Phase Q step 1 with executable SV preprocessor grammar seed
- Root cause:
  - preprocessor-first strategy needed an executable artifact before parser/stimuli/preprocess integration could be hardened.
- Fix:
  - added `grammars/systemverilog_preprocessor.ebnf` with initial directive coverage (`define/undef/include/ifdef family/timescale/default_nettype/celldefine`) plus macro formal/default and token-paste/stringize body primitives.
  - updated `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` Phase Q first item to complete.
  - recorded implementation details in `CHANGES.md` and `DEVELOPMENT_NOTES.md`.
- Validation:
  - `tools/ebnf_to_json.pl --pretty --quiet grammars/systemverilog_preprocessor.ebnf -o /tmp/systemverilog_preprocessor.json`
  - `cargo run --bin ast_pipeline -- /tmp/systemverilog_preprocessor.json --generate-parser --output /tmp/systemverilog_preprocessor_parser.rs --eliminate-left-recursion`
  - `cargo run --bin ast_pipeline -- /tmp/systemverilog_preprocessor.json --generate-stimuli --count 4 --seed 2026 --output /tmp/systemverilog_preprocessor_stimuli.txt`
  - all passed in non-bootstrap flow.

### 2026-02-27: Added explicit SV preprocessor-first closure strategy to roadmap
- Root cause:
  - Nexsim-targeted SV closure needed explicit sequencing; parser/stimuli semantic closure without preprocessing closure would leave a major correctness gap.
- Fix:
  - updated `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` with:
    - hard prerequisite contract: Phase P depends on Phase Q,
    - new `Phase Q` (`SystemVerilog Preprocessor Frontend Closure`) covering dedicated preprocessor grammar, preprocess execution stage, preprocessor quality gate, preprocess-aware stimuli modes, and staged gate policy promotion.
  - mirrored decision in:
    - `CHANGES.md`
    - `DEVELOPMENT_NOTES.md`
- Validation:
  - roadmap/doc sections updated and aligned on the same execution order (`preprocess -> parse_full -> semantic validate`).

### 2026-02-27: Roadmap expansion for Nexsim-targeted SV syntax+semantic closure
- Root cause:
  - roadmap needed an explicit execution phase for SOTA SystemVerilog parser/stimuli hardening where semantic correctness is a first-class acceptance contract, not parseability-only.
- Fix:
  - added Phase P in `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`:
    - codified toolbox baseline (`systemverilog.ebnf`, `ebnf.ebnf`, return/semantic annotation grammars + generated parsers),
    - added explicit SV gate plan (`sv_stimuli_quality_gate`),
    - added syntax closure matrix requirement,
    - added semantic validation profile requirements,
    - added semantic-annotation-driven `sv_snippet`/`sv_file` stimuli modes,
    - enforced closed-loop convergence doctrine (`generate -> parse -> semantic-validate -> coverage/gap feedback`).
- Validation:
  - roadmap updated and changelog entry recorded in roadmap file.

### 2026-02-27: Added initial `systemverilog.ebnf` seed grammar from IEEE markdown
- Root cause:
  - HDL readiness gate had executable skeleton but no HDL grammar file, so both tracked rows were `not_ready`.
- Fix:
  - added `grammars/systemverilog.ebnf` seeded from IEEE 1800 markdown syntax sections (5/23/24/25/26/27),
  - implemented pragmatic initial coverage for top-level design units, declarations/items, expressions/statements, instantiation, and generate constructs,
  - validated through full non-bootstrap frontend pipeline (`EBNF -> JSON -> parser -> stimuli`).
- Validation:
  - `tools/ebnf_to_json.pl --pretty --quiet grammars/systemverilog.ebnf -o /tmp/systemverilog.json`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo run --quiet --bin ast_pipeline -- /tmp/systemverilog.json --generate-parser --output /tmp/systemverilog_parser.rs --eliminate-left-recursion`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo run --quiet --bin ast_pipeline -- /tmp/systemverilog.json --generate-stimuli --count 4 --seed 2026 --output /tmp/systemverilog_stimuli.txt`
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness` now shows:
    - `systemverilog`: `pass`
    - `vhdl`: `not_ready` (pending `grammars/vhdl.ebnf`).

### 2026-02-27: Pillar 5 kickoff (HDL frontend readiness gate skeleton)
- Root cause:
  - roadmap pillar 5 had no executable gate surface and no machine-readable readiness state.
- Fix:
  - added `rust/scripts/hdl_frontend_readiness_gate.sh` with staged checks for `systemverilog` and `vhdl`,
  - added Makefile targets:
    - `make -C rust hdl_frontend_readiness`
    - `make -C rust hdl_frontend_gate`
  - added report/strict semantics:
    - report mode returns `not_ready` rows for missing grammars,
    - strict mode fails on missing/failing flows.
- Validation:
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness` passed (report mode) with expected `not_ready` rows.
  - strict probe confirms fail-until-grammars behavior (`make -C rust SHELL=/bin/bash hdl_frontend_gate`).

### 2026-02-27: First-class tracing + `trace.log` routing baseline
- Root cause:
  - tracing behavior was fragmented across ad-hoc debug prints and local logger paths,
  - no single verbosity model or sink abstraction existed for predictable redirection.
- Fix:
  - added unified `TraceVerbosity`/`TraceLevel` + global sink control in `rust/src/ast_pipeline/mod.rs`,
  - wired runtime parser/stimuli/pipeline logging to trace-aware paths,
  - added CLI controls:
    - `--verbosity <none|low|medium|high|debug>`
    - `--trace-log-file [PATH]` with default `trace.log` when value omitted.
  - enforced trace origin metadata contract:
    - every trace line now includes `file`, `function`, and `line`,
    - function name resolved from runtime backtrace with per-callsite caching.
- Validation:
  - `cd rust && RUSTFLAGS='-Awarnings' cargo check --features generated_parsers,ebnf_dual_run --bins -q`
  - `cd rust && RUSTFLAGS='-Awarnings' cargo run --quiet --bin ast_pipeline -- ../generated/json.json --generate-stimuli --count 1 --verbosity debug --trace-log-file --output /tmp/pgen_stimuli_2.txt`
  - verified trace lines are written to `rust/trace.log` and not emitted as `[PGEN]` lines on stdout.
  - verified trace header format includes `[<file>:<line>] [<function>]`.

### 2026-02-26: EBNF parseability promotion in non-annotation loop
- Root cause:
  - `ebnf` parseability was optional in contract due to missing executable registry path.
- Fix:
  - Added feature-gated `ebnf` parseability adapter in `rust/src/parser_registry.rs`.
  - Promoted `ebnf.require_parseability=true` in `rust/test_data/grammar_quality/ebnf_stimuli_contract.json`.
  - Hardened `rust/scripts/ebnf_stimuli_quality_gate.sh`:
    - bootstrap `generated/ebnf.json` and `generated/ebnf.rs` when required,
    - rebuild `ast_pipeline` with `generated_parsers + ebnf_dual_run`.
- Validation:
  - targeted parser_registry tests passed
  - `PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh` passed.

### 2026-02-26: Dual-run strict promotion to required aggregate policy
- Root cause:
  - Dual-run check was still informational in aggregate policy despite strict gate being green.
- Fix:
  - `rust/config/sota_exit_policy.env` updated:
    - `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
  - Docs synchronized (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `PGEN_USER_GUIDE.md`, roadmap).
- Validation:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate` passed
  - focused `sota_exit_gate` policy-path run passed with dual-run as required.

## Next Likely Tasks (Priority)
1. Plug real trusted-reference adapters into SV preprocessor differential gate:
   - provide project-level runner scripts for available external preprocessors and start collecting taxonomy deltas on curated corpora.
2. Continue Phase P semantic-closure implementation for SV:
   - promote currently optional semantic baseline toggles toward required contract checks once false-positive rate is controlled.
3. Add annotation-driven SV stimuli steering:
   - wire semantic-annotation controls into stimuli branch/value decisions beyond current mode/profile toggles.
4. Expand contractized SV/VHDL corpora:
   - add deterministic targeted families for declaration/use, port binding, generate, and preprocess-heavy cases.
5. Promote VHDL aggregate mode from informational to strict-required when stability criteria are met:
   - keep `PGEN_SOTA_POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT=0` until deterministic pass rate is proven across broader corpus.
6. Continue Rust-native EBNF migration hardening:
   - preserve parity/dual-run contracts while reducing Perl frontend dependence.
7. Keep roadmap + UG + memory synced after every gate/contract increment.

## Known Gaps / Risks
- Pipeline is still hybrid (`ebnf_to_json.pl` remains active in core/gate flows).
- Rust EBNF frontend exists and is validated via dual-run, but is not full replacement yet.
- Semantic-annotation leverage in SV/VHDL stimuli generation is still partial; mode-level policy exists but full directive-driven steering is not closed yet.
- Aggregate VHDL stimuli gate is currently informational-first; strict promotion is pending additional stability evidence.
- SV preprocessor differential taxonomy stage is wired, but project-level trusted-reference runners are not yet standardized.

## Quick Commands
- HDL frontend readiness (report):
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness`
- HDL frontend readiness (strict):
  - `make -C rust SHELL=/bin/bash hdl_frontend_gate`
- Strict dual-run check:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
- Non-annotation closed-loop quality:
  - `PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh`
- VHDL closed-loop quality:
  - `make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate`
- SV preprocessor strict differential example:
  - `PGEN_SV_PREPROCESSOR_DIFF_MODE=1 PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER=/abs/path/to/runner.sh make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- Aggregate gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
