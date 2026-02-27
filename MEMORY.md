# MEMORY.md

Last updated: 2026-02-27 (+0100)

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
- Worktree: dirty (pending roadmap/docs updates for SV preprocessor-first execution contract; run `git status -sb`).
- Latest commit: see tail entry in "Session Git History (Hash + Message)".
- SOTA policy status:
  - strict EBNF readiness required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`
  - strict EBNF dual-run required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
- Non-annotation parseability contract:
  - `ebnf` is now `require_parseability=true` (with `ebnf_dual_run` adapter path).

## Session Git History (Hash + Message)
- Scope used for continuity tracking: `origin/main..HEAD`
- Commit count at last refresh (before current uncommitted changes): `135`
- Refresh command:
  - `git log --oneline --reverse origin/main..HEAD`
<!-- SESSION_GIT_HISTORY_BEGIN -->
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
1. Start Phase P execution item 1:
   - create `systemverilog_core_v0` contract corpus and add `sv_stimuli_quality_gate` skeleton.
2. Add first VHDL seed grammar:
   - `grammars/vhdl.ebnf`
   - drive `make -C rust hdl_frontend_gate` to green for both tracked HDL grammars.
3. Decide SOTA aggregate integration path for HDL readiness:
   - informational first in `sota_exit_gate`, then required strict when seed grammars stabilize.
4. Continue Rust-native EBNF migration hardening:
   - reduce reliance on Perl frontend where safe, while preserving strict parity gates.
5. Expand parser-registry coverage beyond annotations/ebnf:
   - onboard `json` and `regex` parseability adapters once generated parser integration path is stable.
6. Keep User Guide expansion in sync with advanced steering/gate behavior and operator workflows.

## Known Gaps / Risks
- Pipeline is still hybrid (`ebnf_to_json.pl` remains active in core/gate flows).
- Rust EBNF frontend exists and is validated via dual-run, but is not full replacement yet.
- Pillar 5 is now started and `systemverilog.ebnf` is present/flowing; `vhdl.ebnf` is still missing, so strict HDL readiness is expected to fail.

## Quick Commands
- HDL frontend readiness (report):
  - `make -C rust SHELL=/bin/bash hdl_frontend_readiness`
- HDL frontend readiness (strict):
  - `make -C rust SHELL=/bin/bash hdl_frontend_gate`
- Strict dual-run check:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
- Non-annotation closed-loop quality:
  - `PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh`
- Aggregate gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
