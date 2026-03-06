# MEMORY.md

Last updated: 2026-03-06 (+0100, task: phase-p-declared-identifier-indexed-lhs-contract-hardening)

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
- Worktree: verify with `git status -sb` before resuming; commit workflow is required after each completed task.
- Latest commit: `86910f3` (`Add configurable seed-stride control to SV parse-full promotion trials.`).
- SOTA policy status:
  - strict EBNF readiness required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`
  - strict EBNF dual-run required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
  - SV parse-full ratio promotion stage enabled informationally:
    - `PGEN_SOTA_POLICY_RUN_SV_PARSE_FULL_RATIO_PROMOTION=1`
    - `PGEN_SOTA_POLICY_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT=0`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS=4`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT=8`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE=12001`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE=250000`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE=auto`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE=0`
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE=sv_file`
  - SV declared-shadow promotion stage strict-enabled with policy-driven trial shape:
    - `PGEN_SOTA_POLICY_RUN_SV_DECLARED_SHADOW_PROMOTION=1`
    - `PGEN_SOTA_POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT=1`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TRIALS=3`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_COUNT=6`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE=12001`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS=400`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE=auto`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED=2`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE=1`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE=sv_file`
    - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY=1`
  - Declared-shadow promotion diagnostics parity:
    - report now includes blocker taxonomy (`trials[].blocker_key/detail`, `blockers.*`),
    - aggregate `sota_exit_gate` now surfaces:
      - `sv_declared_shadow_promotion_primary_non_shadow_blocker`.
  - Declared-shadow promotion parseability scope is now policy-driven in standalone + aggregate gate paths:
    - standalone knob:
      - `PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY`
    - aggregate policy/runtime knobs:
      - `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY`
      - `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY`
  - Aggregate declared-shadow promotion telemetry now also surfaces runtime-effective stage scope:
    - `sv_declared_shadow_promotion_declared_shadow_parseable_only`
  - Aggregate declared-shadow promotion telemetry now also surfaces blocker counters:
    - `sv_declared_shadow_promotion_failed_trial_count`
    - `sv_declared_shadow_promotion_non_shadow_blocked_trial_count`
  - Aggregate parse-full promotion telemetry now also surfaces blocker counters:
    - `sv_parse_full_ratio_promotion_failed_trial_count`
    - `sv_parse_full_ratio_promotion_non_ratio_blocked_trial_count`
  - Aggregate parse-full promotion telemetry now also surfaces observed ratio range:
    - `sv_parse_full_ratio_promotion_observed_ratio_min`
    - `sv_parse_full_ratio_promotion_observed_ratio_max`
  - Aggregate `sv_stimuli_quality_gate` now runs under aggregate state and emits core telemetry:
    - stage dir: `rust/target/sota_exit_gate/work/sv_stimuli_quality_gate`
    - `sv_stimuli_quality_parse_full_pass_ratio_percent`
    - `sv_stimuli_quality_diff_mismatch_count`
    - `sv_stimuli_quality_performance_enabled`
  - Aggregate SV parse-full strict floor now set to:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=100`
  - Aggregate `sv_preprocessor_quality_gate` now runs under aggregate state and emits differential telemetry:
    - stage dir: `rust/target/sota_exit_gate/work/sv_preprocessor_quality_gate`
    - `sv_preprocessor_quality_parseability_mode_effective`
    - `sv_preprocessor_quality_diff_mode_effective`
    - `sv_preprocessor_quality_diff_mismatch_count`
    - key taxonomy counters (`output_mismatch`, `rust_failed_reference_passed`, `reference_failed_rust_passed`)
  - New offline curated preprocessor differential gate is available:
    - target: `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate`
    - corpus: `rust/test_data/grammar_quality/systemverilog_preprocessor_curated_differential_corpus.json`
    - no external `iverilog`/`verilator` dependency
    - classification buckets:
      - `expected_match`
      - `expected_mismatch`
      - `bug_mismatch`
    - corpus expanded to 9 cases (`version: 4`) with explicit contract split:
      - 7 stable positive families: `expected_categories: ["match"]`
      - 2 deterministic include-policy negatives: `expected_categories: ["rust_failed_expected_passed"]`
  - New dynamic template preprocessor differential gate is available:
    - target: `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate`
    - no external `iverilog`/`verilator` dependency
    - deterministic seed-driven template synthesis with predicted outputs/diagnostics
    - template families:
      - `template_define_width`
      - `template_ifdef_branch`
      - `template_token_paste`
      - `template_define_undef_ifdef`
    - classification buckets:
      - `expected_match`
      - `expected_mismatch`
      - `bug_mismatch`
  - Dynamic template gate now includes expanded edge templates + diagnostics contracts:
    - additional templates:
      - `template_nested_ifdef`
      - `template_macro_function_args`
    - diagnostics invariants:
      - observed diagnostics must be JSON-array shaped,
      - warning/error counts must match expected per-case invariant,
      - aggregate counters:
        - `diagnostics_invariant_pass_count`
        - `diagnostics_invariant_fail_count`
- Non-annotation parseability contract:
  - `ebnf` is now `require_parseability=true` (with `ebnf_dual_run` adapter path).
- SV semantic deterministic suite baseline:
  - enforced suite manifests now at `version: 2` for:
    - `systemverilog_declared_identifier_contract_cases.json`
    - `systemverilog_package_qualification_contract_cases.json`
    - `systemverilog_width_compatibility_contract_cases.json`
    - `systemverilog_context_legality_contract_cases.json`
    - `systemverilog_port_binding_legality_contract_cases.json`
  - suites now include preprocess-heavy directive families (macro/conditional noise) in addition to plain snippet families.
- SV grammar semantic steering baseline:
  - initial annotation-driven steering directives are now embedded in `grammars/systemverilog.ebnf`:
    - top-level coverage hints (`@coverage_target`, `@critical_path`),
    - branch steering (`@branch_policy`, `@priority`) on `source_item`, `description`, and `statement`,
    - token-family hints (`@token_class`) on `simple_identifier` and `integral_number`.
  - rule-level expansion now extends steering to additional high-fanout rules:
    - item/declaration/procedural fanout:
      - `module_item`, `non_port_module_item`, `program_item`, `non_port_program_item`,
      - `module_or_generate_item`, `interface_or_generate_item`, `package_or_generate_item_declaration`,
      - `generate_item`, `block_item_declaration`, `statement_or_null`.
    - lexical/shape steering:
      - `module_keyword`, `module_header_ports`, `named_port_connection`, `hierarchy_separator`,
      - `primary`, `data_type`, `identifier` (favoring `simple_identifier`).
  - parse-full burn-down subset mode is now wired:
    - grammar entry: `systemverilog_parseable_file := parseable_source_item*`,
    - contract mode: `sv_parseable_file` (`entry_rule=systemverilog_parseable_file`, parse-full eligible, closed-loop enabled),
    - latest deterministic gate evidence:
      - `PGEN_SV_STIMULI_QUALITY_MODE=sv_parseable_file ... sv_stimuli_quality_gate`
      - `parse_full_pass_ratio_percent=100` (`12/12` across `2017` + `2023` profiles).
  - `sv_file` burn-down now includes generator-side boundary-spacing control + mode profile generation caps:
    - new CLI control: `--enforce-word-boundary-spacing`,
    - wired in `sv_stimuli_quality_gate` generation paths,
    - sequence/quantified segment concatenation now also enforces lexical boundary spacing under the same flag,
    - `sv_file` profile now uses contractized caps:
      - `max_depth=20`
      - `max_repeat=2`
    - deterministic `sv_file` evidence now reaches `100%` parse-full pass ratio (`12/12`) in dual-profile run.

### 2026-03-06: Broadened ceiling parse-full promotion evidence density from 3x6 to 4x8
- Root cause:
  - ceiling policy (`min=100`, `target=100`) was green under `3x6`, but sustained-ceiling confidence needed denser deterministic promotion evidence before broad corpus expansions.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS=4` (from `3`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT=8` (from `6`).
  - kept ceiling acceptance policy unchanged:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=100`,
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100`.
- Validation:
  - wider strict quality run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_COUNT=8 PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=100 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`16/16`).
  - denser promotion run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100 PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS=4 PGEN_SV_PARSE_FULL_RATIO_PROMOTION_COUNT=8 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=4/4`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate promotion telemetry is now backed by denser deterministic evidence while ceiling floor/target semantics remain unchanged.

### 2026-03-06: Promoted SV closed-loop contract defaults from 6/6 to 8/8 (`systemverilog_core_v0` v24)
- Root cause:
  - wider deterministic stress (`8` sample shape) had already been proven through overrides, but default contract settings remained `6/6`, so broader evidence was not baseline behavior.
- Fix:
  - updated `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`:
    - `version: 24` (from `23`),
    - `sample_count: 8` (from `6`),
    - `closed_loop.replay_sample_count: 8` (from `6`).
- Validation:
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - strict contract-default run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=100 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`16/16`), `sample_count=8`, and `closed_loop_replay_sample_count=8`.
  - sustained ceiling promotion run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100 PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS=4 PGEN_SV_PARSE_FULL_RATIO_PROMOTION_COUNT=8 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=4/4`, observed ratio `100/100/100`.
- Result:
  - broader SV stress is now baseline contract behavior rather than override-only evidence.

### 2026-03-06: Ratcheted aggregate parse-full floor from 95 to 100 with promotion target held at ceiling
- Root cause:
  - deterministic strict evidence remained fully green at threshold `100`, so aggregate floor `95` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=100` (from `95`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100` (unchanged ceiling target).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=100 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - ceiling-target promotion run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `100`; promotion target remains at `100` for sustained-ceiling evidence.

### 2026-03-05: Ratcheted aggregate parse-full floor from 90 to 95 and advanced promotion target to 100
- Root cause:
  - deterministic strict evidence remained fully green at threshold `95`, so aggregate floor `90` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=95` (from `90`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100` (from `95`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=95 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=100 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `95` and tracks next-ratchet evidence at `100`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 85 to 90 and advanced promotion target to 95
- Root cause:
  - deterministic strict evidence remained fully green at threshold `90`, so aggregate floor `85` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=90` (from `85`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=95` (from `90`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=90 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=95 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `90` and tracks next-ratchet evidence at `95`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 80 to 85 and advanced promotion target to 90
- Root cause:
  - deterministic strict evidence remained fully green at threshold `85`, so aggregate floor `80` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=85` (from `80`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=90` (from `85`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=85 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=90 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `85` and tracks next-ratchet evidence at `90`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 75 to 80 and advanced promotion target to 85
- Root cause:
  - deterministic strict evidence remained fully green at threshold `80`, so aggregate floor `75` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=80` (from `75`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=85` (from `80`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=80 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=85 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `80` and tracks next-ratchet evidence at `85`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 70 to 75 and advanced promotion target to 80
- Root cause:
  - deterministic strict evidence remained fully green at threshold `75`, so aggregate floor `70` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=75` (from `70`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=80` (from `75`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=75 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=80 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `75` and tracks next-ratchet evidence at `80`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 65 to 70 and advanced promotion target to 75
- Root cause:
  - deterministic strict evidence remained fully green at threshold `70`, so aggregate floor `65` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=70` (from `65`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=75` (from `70`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=70 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=75 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `70` and tracks next-ratchet evidence at `75`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 60 to 65 and advanced promotion target to 70
- Root cause:
  - deterministic strict evidence remained fully green at threshold `65`, so aggregate floor `60` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=65` (from `60`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=70` (from `65`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=65 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=70 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `65` and tracks next-ratchet evidence at `70`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 55 to 60 and advanced promotion target to 65
- Root cause:
  - deterministic strict evidence remained fully green at threshold `60`, so aggregate floor `55` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=60` (from `55`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=65` (from `60`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=60 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=65 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `60` and tracks next-ratchet evidence at `65`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 50 to 55 and advanced promotion target to 60
- Root cause:
  - deterministic strict evidence remained fully green at threshold `55`, so aggregate floor `50` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=55` (from `50`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=60` (from `55`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=55 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=60 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `55` and tracks next-ratchet evidence at `60`.

### 2026-03-05: Ratcheted aggregate parse-full floor from 45 to 50 and advanced promotion target to 55
- Root cause:
  - deterministic strict evidence remained fully green at threshold `50`, so aggregate floor `45` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=50` (from `45`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=55` (from `50`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO=1 PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=50 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=55 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `50` and tracks next-ratchet evidence at `55`.

### 2026-03-04: Closed current `sv_file` parse-full burn-down increment with segment-boundary spacing + mode caps
- Root cause:
  - terminal-`\b` spacing fix reduced debt but generated sequence/quantified fragments could still fuse lexical words via raw concatenation, and mode-level generation caps declared in contract were not yet enforced in gate invocations.
- Fix:
  - `StimuliGenerator` now uses lexical-boundary-safe `append_generated_segment(...)` in sequence/quantified generation paths when `enforce_word_boundary_spacing` is enabled.
  - `sv_stimuli_quality_gate` now reads/validates/forwards:
    - `stimuli_modes.profiles.<mode>.max_depth`
    - `stimuli_modes.profiles.<mode>.max_repeat`
    across initial/replay/per-sample generation phases.
  - `systemverilog_core_v0_contract.json` advanced to `version: 23` with `sv_file.max_depth=20`, `sv_file.max_repeat=2`.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml word_boundary_spacing_policy_appends_separator_for_terminal_boundary` passed.
  - `cargo test --manifest-path rust/Cargo.toml word_spacing_policy_separates_adjacent_word_segments_in_sequences` passed.
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... sv_stimuli_quality_gate` passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - `make -C rust SHELL=/bin/bash clippy_on_rust_change` passed.
- Result:
  - deterministic `sv_file` parse-full burn-down lane is currently fully green for this trial shape while semantic deterministic suites remain green.

### 2026-03-04: Ratcheted aggregate parse-full floor from 15 to 20 and advanced promotion target to 25
- Root cause:
  - aggregate required threshold remained at `15` even though deterministic `sv_file` burn-down evidence had converged well above that floor.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=20` (from `15`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=25` (from `20`) to keep promotion stage focused on next ratchet.
- Validation:
  - `make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate` at target `20`:
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=20 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=25 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`.
- Result:
  - aggregate policy floor is now aligned with deterministic evidence, and promotion telemetry now tracks the next ratchet candidate.

### 2026-03-04: Ratcheted aggregate parse-full floor from 20 to 25 and advanced promotion target to 30
- Root cause:
  - deterministic strict evidence remained fully green at threshold `25`, so aggregate floor `20` was stale and no longer reflected proven quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=25` (from `20`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=30` (from `25`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=25 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=30 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces a stronger required parse-full floor (`25`) and tracks next-ratchet evidence at `30`.

### 2026-03-04: Ratcheted aggregate parse-full floor from 25 to 30 and advanced promotion target to 35
- Root cause:
  - deterministic strict evidence remained fully green at threshold `30`, so aggregate floor `25` was stale relative to measured quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=30` (from `25`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=35` (from `30`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=30 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=35 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `30` and tracks next-ratchet evidence at `35`.

### 2026-03-04: Ratcheted aggregate parse-full floor from 30 to 35 and advanced promotion target to 40
- Root cause:
  - deterministic strict evidence remained fully green at threshold `35`, so aggregate floor `30` was stale relative to objective quality evidence.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=35` (from `30`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=40` (from `35`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=35 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=40 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `35` and tracks next-ratchet evidence at `40`.

### 2026-03-04: Ratcheted aggregate parse-full floor from 35 to 40 and advanced promotion target to 45
- Root cause:
  - deterministic strict evidence remained fully green at threshold `40`, so aggregate floor `35` was stale relative to measured acceptance quality.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=40` (from `35`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=45` (from `40`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=40 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=45 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `40` and tracks next-ratchet evidence at `45`.

### 2026-03-04: Ratcheted aggregate parse-full floor from 40 to 45 and advanced promotion target to 50
- Root cause:
  - deterministic strict evidence remained fully green at threshold `45`, so aggregate floor `40` was stale relative to objective acceptance evidence.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=45` (from `40`),
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=50` (from `45`).
- Validation:
  - strict threshold run:
    - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO=45 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - passed with `parse_full_pass_ratio_percent=100` (`12/12`).
  - informational next-target run:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO=50 make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate`
    - `trial_passed=3/3`, recommendation `raise_min_parse_full_pass_ratio`, observed ratio `100/100/100`.
- Result:
  - aggregate policy now enforces required parse-full floor `45` and tracks next-ratchet evidence at `50`.

### 2026-03-03: Added generator word-boundary spacing control and SV gate wiring
- Root cause:
  - generated SV stimuli frequently fused terminal-word-boundary regex tokens with following word tokens, causing parse_full rejections despite grammar-valid intent.
- Fix:
  - added `StimuliConfig.enforce_word_boundary_spacing` and CLI flag `--enforce-word-boundary-spacing`,
  - when enabled, terminal `\\b` regex samples append delimiter space if candidate ends in word char,
  - enabled the flag in all `sv_stimuli_quality_gate` generation invocations (closed-loop + per-sample).
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml word_boundary_spacing_policy_appends_separator_for_terminal_boundary` passed.
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_file ... sv_stimuli_quality_gate` passed with improved parse-full telemetry (`41%`).
  - `make -C rust SHELL=/bin/bash clippy_on_rust_change` passed.
- Result:
  - parse-full debt on `sv_file` reduced materially while keeping semantic deterministic suites green.

### 2026-03-03: Added SV parseable-subset mode for parse-full burn-down evidence
- Root cause:
  - parse-full telemetry on broad SV mode remained debt-heavy/variable despite steering expansion.
- Fix:
  - added parseable-subset grammar path in `grammars/systemverilog.ebnf`:
    - `systemverilog_parseable_file`
    - `parseable_source_item`
  - added whitespace/trivia steering for cleaner generated corpora:
    - `trivia` branch priorities now favor `white_space`,
    - `white_space` now carries `@token_class: whitespace` + `@enum: [" "]`.
  - added new contract mode in `systemverilog_core_v0_contract.json` (`version: 22`):
    - `sv_parseable_file` with parse-full-eligible closed-loop execution.
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_MODE=sv_parseable_file PGEN_SV_STIMULI_QUALITY_COUNT=6 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed.
- Result:
  - parse-full pass ratio reached `100%` (`12/12`) in this mode with all deterministic semantic suites still green.

### 2026-03-03: Added aggregate SV preprocessor quality artifact scoping + telemetry
- Root cause:
  - Aggregate `sota_exit_gate` executed `sv_preprocessor_quality_gate` without aggregate-scoped state routing and without surfaced preprocessor differential telemetry.
- Fix:
  - Updated `rust/scripts/sota_exit_gate.sh` to:
    - route stage artifacts under `rust/target/sota_exit_gate/work/sv_preprocessor_quality_gate`,
    - forward `PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR`,
    - surface effective mode and mismatch/taxonomy telemetry in stdout + summary.
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh` passed.
  - focused aggregate run (required checks limited to `differential_baseline_contract`, preprocessor stage enabled) passed and emitted telemetry fields.
- Result:
  - Aggregate triage for preprocessor differential runs no longer requires manual drill-down into stage-local files.

### 2026-03-03: Added offline curated SV preprocessor differential gate
- Root cause:
  - Differential taxonomy hardening needed deterministic expected-vs-bug classification without relying on host-installed external preprocessors.
- Fix:
  - Added `rust/scripts/sv_preprocessor_curated_differential_gate.sh`:
    - compares `--preprocess-systemverilog` output and diagnostics against checked-in expected artifacts from curated corpus.
    - supports `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=auto|0|1`.
    - strict mode fails only on `bug_mismatch`.
  - Added curated corpus + expected artifacts under:
    - `rust/test_data/grammar_quality/systemverilog_preprocessor_curated/`
    - `rust/test_data/grammar_quality/systemverilog_preprocessor_curated_differential_corpus.json`
  - Added Make target:
    - `sv_preprocessor_curated_differential_gate`.
- Validation:
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=auto bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=1 bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate` passed.
- Result:
  - Curated preprocessor differential classification is now deterministic and fully offline.

### 2026-03-03: Added dynamic template-based SV preprocessor differential gate
- Root cause:
  - Purely static corpora do not scale for broad automated differential evidence; we needed deterministic dynamic generation/prediction without external tool dependencies.
- Fix:
  - Added `rust/scripts/sv_preprocessor_template_differential_gate.sh`:
    - generates deterministic SV snippets from seed-driven templates,
    - predicts expected output/diagnostics offline via template logic,
    - compares runtime output vs predicted expected artifacts,
    - classifies `expected_match`/`expected_mismatch`/`bug_mismatch`.
  - Added Make target:
    - `sv_preprocessor_template_differential_gate`.
  - Added strict behavior:
    - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE=1` fails only when `bug_mismatch_count > 0`.
- Validation:
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE=auto bash rust/scripts/sv_preprocessor_template_differential_gate.sh` passed.
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE=1 bash rust/scripts/sv_preprocessor_template_differential_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate` passed.
- Result:
  - Differential automation now scales dynamically while remaining deterministic and offline.

### 2026-03-03: Expanded dynamic template differential coverage + diagnostics invariants
- Root cause:
  - Initial dynamic gate covered baseline templates but did not include nested conditional/macro-arg edge cases and lacked explicit diagnostics shape/count invariants.
- Fix:
  - Expanded `rust/scripts/sv_preprocessor_template_differential_gate.sh` with:
    - `template_nested_ifdef`
    - `template_macro_function_args`
  - Added diagnostics contract checks and telemetry:
    - taxonomy: `diagnostics_contract_violation`
    - counters: `diagnostics_invariant_pass_count`, `diagnostics_invariant_fail_count`
    - per-case diagnostics invariant envelope in report JSON.
- Validation:
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE=auto PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_COUNT=24 bash rust/scripts/sv_preprocessor_template_differential_gate.sh` passed.
  - `PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE=1 PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_COUNT=24 bash rust/scripts/sv_preprocessor_template_differential_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate` passed.
- Result:
  - Dynamic differential coverage is broader and diagnostics behavior is now contractized with objective failure attribution.

### 2026-03-03: Expanded curated offline corpus and tightened to match-only contracts
- Root cause:
  - Curated corpus still had only 3 cases and tolerated whitespace-only mismatches for stable families.
- Fix:
  - Expanded curated corpus manifest to `version: 3` with 7 cases by adding:
    - `macro_define_undef_guard`
    - `nested_conditionals`
    - `macro_function_args`
    - `include_local_file` (with local include fixture `include_payload.svh`)
  - Regenerated expected artifacts (`*.expected.sv`, `*.expected.diag.json`) deterministically from current preprocessor behavior.
  - Tightened all curated entries to `expected_categories: ["match"]`.
- Validation:
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=auto bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=1 bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate` passed.
- Result:
  - Curated offline differential gate now enforces strict byte-level/dataset-level conformance over broader directive coverage.

### 2026-03-03: Expanded curated offline corpus with deterministic include-policy negatives
- Root cause:
  - Curated corpus covered positive directive families but lacked deterministic negative include-policy/failure families.
- Fix:
  - Advanced curated corpus manifest to `version: 4` and added:
    - `include_missing_file_negative`
    - `include_cycle_negative` (with fixtures `include_cycle_a.svh`, `include_cycle_b.svh`)
  - Added explicit expected-failure contracts for those families:
    - `expected_categories: ["rust_failed_expected_passed"]`
  - Preserved strict `match` contracts for the 7 stabilized positive families.
- Validation:
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=auto bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE=1 bash rust/scripts/sv_preprocessor_curated_differential_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate` passed.
  - strict summary counters:
    - `diff_cases_declared=9`
    - `classification_expected_match=7`
    - `classification_expected_mismatch=2`
    - `classification_bug_mismatch=0`
- Result:
  - Curated offline differential coverage now spans both stable positive conformance and deterministic include-policy failure families with zero bug mismatches.

### 2026-03-03: Expanded deterministic SV semantic contract suites with preprocess-heavy families
- Root cause:
  - Phase P semantic closure lacked deterministic suite coverage for preprocess-shaped directive patterns.
- Fix:
  - Updated enforced semantic suite manifests to `version: 2`:
    - `systemverilog_declared_identifier_contract_cases.json`
    - `systemverilog_package_qualification_contract_cases.json`
    - `systemverilog_width_compatibility_contract_cases.json`
    - `systemverilog_context_legality_contract_cases.json`
    - `systemverilog_port_binding_legality_contract_cases.json`
  - Added directive-heavy deterministic cases (macro-qualified references, conditional-directive families, directive-noise context/port patterns).
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed.
  - Suite counters:
    - `declared_identifier_suite_passed=14/14`
    - `width_compatibility_suite_passed=10/10`
    - `port_binding_suite_passed=10/10`
    - `package_qualification_suite_passed=10/10`
    - `context_legality_suite_passed=10/10`
- Result:
  - Deterministic semantic closure evidence now covers preprocess-heavy patterns in addition to plain snippet families.

### 2026-03-03: Started annotation-driven SV stimuli steering rollout in systemverilog grammar
- Root cause:
  - SV grammar flow still lacked embedded semantic steering directives despite stimuli engine support.
- Fix:
  - Added initial semantic steering annotations in `grammars/systemverilog.ebnf`:
    - top-level coverage hints:
      - `@coverage_target: 4`
      - `@critical_path: true`
    - branch steering:
      - `source_item` (`@branch_policy: priority_first`, `@priority: [12,3,2,1,1]`)
      - `description` (`@branch_policy: priority_first`, `@priority: [12,4,3,2,1]`)
      - `statement` (`@branch_policy: priority_first`, `@priority: [10,5,4,3,3,2,8,8,3,2,1]`)
    - token-family hints:
      - `simple_identifier` (`@token_class: identifier`)
      - `integral_number` (`@token_class: integer`)
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed.
  - semantic suites remained green:
    - declared identifier `14/14`
    - width `10/10`
    - port binding `10/10`
    - package qualification `10/10`
    - context legality `10/10`
- Result:
  - Annotation-driven SV steering is now active at grammar level with a deterministic initial baseline; expansion to broader rule coverage remains pending.

### 2026-03-03: Expanded annotation-driven SV steering to additional high-fanout rules
- Root cause:
  - Initial steering baseline covered only a small subset of SV rules and did not yet shape many high-fanout generation paths.
- Fix:
  - Extended semantic steering directives in `grammars/systemverilog.ebnf` to additional rules:
    - fanout/item rules: `module_item`, `non_port_module_item`, `program_item`, `non_port_program_item`, `module_or_generate_item`, `interface_or_generate_item`, `package_or_generate_item_declaration`, `generate_item`, `block_item_declaration`, `statement_or_null`.
    - lexical/shape rules: `module_keyword`, `module_header_ports`, `named_port_connection`, `hierarchy_separator`, `primary`, `data_type`, `identifier`.
  - Corrected identifier steering priority to favor `simple_identifier` over `escaped_identifier`.
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed.
  - Semantic suites remained green (`declared_identifier 14/14`, `width 10/10`, `port_binding 10/10`, `package_qualification 10/10`, `context_legality 10/10`).
  - Parse-full telemetry in this run remained `0%`.
  - Result:
  - Steering coverage is materially broader while preserving deterministic gate stability; parse-full debt remains an open closure target.

### 2026-03-06: Expanded declared-identifier suite and fixed indexed-LHS undeclared detection
- Root cause:
  - declared-identifier suite expansion exposed a false-negative: `arr[idx] = ...` did not flag undeclared `idx`.
- Fix:
  - `systemverilog_declared_identifier_contract_cases.json` promoted to `version: 3` with:
    - `indexed_assignment_declared_index_pass`
    - `indexed_assignment_undeclared_index_fail`
  - `check_declared_identifiers_before_use` in `sv_stimuli_quality_gate.sh` now extracts/scans indexed LHS expressions and marks index identifiers as uses.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=2 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=auto make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed.
  - deterministic suite metrics:
    - declared identifier suite `16/16` pass (up from `14/14`),
    - parse-full ratio `100` (`4/4`) in representative run.

### 2026-03-06: Added policy-driven seed-stride hardening for SV parse-full promotion trials
- Root cause:
  - promotion trials used a fixed hardcoded seed stride, so seed-space broadening was implicit and not policy-controlled.
- Fix:
  - added `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE` in standalone promotion gate.
  - added aggregate policy/runtime controls:
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE`
    - `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE`
  - set tracked policy default:
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_STRIDE=250000`
  - surfaced `seed_stride` in standalone and aggregate telemetry.
- Validation:
  - syntax checks passed for `sv_parse_full_ratio_promotion_gate.sh` and `sota_exit_gate.sh`.
  - promotion gate passed at default and target ceiling (`100`) with `trials=4`, `count=8`, observed ratio `100/100/100`.

## Session Git History (Hash + Message)
- Scope used for continuity tracking: `origin/main..HEAD`
- Commit count at last refresh (before current uncommitted changes): `254`
- Refresh command:
  - `git log --oneline --reverse origin/main..HEAD`
<!-- SESSION_GIT_HISTORY_BEGIN -->
- 86910f3 Add configurable seed-stride control to SV parse-full promotion trials.
- f32eb49 Promote SV closed-loop contract defaults from 6/6 to 8/8 (v24)
- 9b88ff4 Increase SV parse-full promotion evidence density from 3x6 to 4x8
- 9de2614 Ratchet aggregate SV parse-full policy floor from 95 to 100
- 971f7b9 Ratchet aggregate SV parse-full policy floor from 90 to 95
- f1a46e6 Ratchet aggregate SV parse-full policy floor from 85 to 90
- e39be92 Ratchet aggregate SV parse-full policy floor from 80 to 85
- 6af3925 Ratchet aggregate SV parse-full policy floor from 75 to 80
- 40b5308 Ratchet aggregate SV parse-full policy floor from 70 to 75
- 622b39d Ratchet aggregate SV parse-full policy floor from 65 to 70
- 4b1afe9 Ratchet aggregate SV parse-full policy floor from 60 to 65
- 6db01d6 Ratchet aggregate SV parse-full policy floor from 55 to 60
- 110e2eb Ratchet aggregate SV parse-full policy floor from 50 to 55
- c48a715 Ratchet aggregate SV parse-full policy floor from 45 to 50
- 16fba51 Ratchet aggregate SV parse-full policy floor from 40 to 45
- ee7b10a Ratchet aggregate SV parse-full policy floor from 35 to 40
- 5877045 Ratchet aggregate SV parse-full policy floor from 30 to 35
- d793f5a Ratchet aggregate SV parse-full policy floor from 25 to 30
- fc0bf0b Ratchet aggregate SV parse-full policy floor from 20 to 25
- 125db14 Ratchet aggregate SV parse-full policy floor from 15 to 20
- 50ba3e2 Harden SV stimuli segment spacing and mode profile generation caps
- 805463c Add word-boundary spacing control for SV stimuli and improve sv_file parse-full
- fa919f8 Add sv_parseable_file mode and parseable-subset SV steering for parse-full burn-down
- e81101f Expand annotation-driven SV stimuli steering to additional high-fanout grammar rules.
- a1c7bbc Start annotation-driven SV stimuli steering rollout via semantic directives in systemverilog.ebnf.
- f98cda9 Expand deterministic SV semantic contract suites with preprocess-heavy directive families.
- d5b4895 Expand curated SV preprocessor differential corpus with include-policy negative families and close Phase Q differential hardening.
- 9e4c155 Expand dynamic SV preprocessor template differential gate with additional edge templates and diagnostics contract invariants.
- 0dd06ee Add dynamic template-based offline SV preprocessor differential oracle gate.
- 302832e Add offline curated SV preprocessor differential gate with expected-artifact oracle and taxonomy classification.
- 4c034c1 Scope aggregate sv_preprocessor_quality_gate artifacts under sota_exit_gate state and surface preprocessor differential telemetry.
- ef6acaa Scope aggregate sv_stimuli_quality_gate artifacts under sota_exit_gate state and surface core telemetry.
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
  2. when Rust/generated Rust files changed, run:
     - `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`
  3. stage intended tracked files only
  4. `git commit -F git_message_brief.txt`
  5. clear `git_message_brief.txt` to 0 bytes
  6. keep `git_message_brief.txt` untracked
- Clippy policy:
  - source lint (`cargo clippy --all-targets`) is strict and must pass.
  - generated-parser lint (`cargo clippy --all-targets --features generated_parsers,ebnf_dual_run`) is always executed by the flow; set `PGEN_CLIPPY_GENERATED_STRICT=1` to make generated lint debt fail the workflow.
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

### 2026-03-01: Added aggregate policy/telemetry parity for declared-shadow promotion stage
- Root cause:
  - declared-shadow promotion stage lacked aggregate policy-driven trial-shape controls and did not emit persisted stage telemetry in aggregate summary artifacts.
- Fix:
  - added `PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_*` + `PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_*` controls for trial-shape tuning,
  - wired validation + forwarding in `sota_exit_gate`,
  - routed stage artifacts under aggregate state and emitted declared-shadow telemetry in stdout and `summary.txt`.
- Validation:
  - focused aggregate run with declared-shadow stage enabled and explicit trial-shape overrides passed,
  - aggregate output and summary included declared-shadow telemetry fields (report path/recommendation/eligibility/failed/checked).

### 2026-03-01: Persisted promotion telemetry into aggregate summary artifact
- Root cause:
  - promotion telemetry was visible in live aggregate output but absent from persisted `summary.txt`, weakening CI artifact handoff.
- Fix:
  - `sota_exit_gate` now stores promotion telemetry fields and appends a `Promotion Telemetry` section to `rust/target/sota_exit_gate/summary.txt` when promotion stage runs.
- Validation:
  - focused aggregate run passed,
  - `summary.txt` contained report path, recommendation, primary non-ratio blocker, and observed ratio average.

### 2026-03-01: Added aggregate-scoped promotion artifacts and inline recommendation telemetry
- Root cause:
  - aggregate promotion stage pass/fail was visible, but recommendation/blocker telemetry required manual log/report discovery outside aggregate output.
- Fix:
  - forced promotion stage state dir under aggregate gate state tree:
    - `rust/target/sota_exit_gate/work/sv_parse_full_ratio_promotion_gate`
  - added aggregate output lines:
    - `sv_parse_full_ratio_promotion_report_json`
    - `sv_parse_full_ratio_promotion_recommendation`
    - `sv_parse_full_ratio_promotion_primary_non_ratio_blocker`
    - `sv_parse_full_ratio_promotion_observed_ratio_avg`
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh` passed,
  - focused aggregate run passed and emitted all new telemetry lines with aggregate-scoped report path.

### 2026-03-01: Made aggregate parse-full promotion trial shape policy-driven
- Root cause:
  - aggregate promotion stage controlled target threshold, but trial shape still depended on script defaults and could not be centrally tuned under aggregate policy.
- Fix:
  - added aggregate policy/runtime knobs for:
    - trials/count/seed_base,
    - parse_full_mode/semantic_closure_mode/stimuli_mode,
  - added validation and forwarding of effective values in `sota_exit_gate` for promotion-stage invocations.
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh` passed,
  - focused aggregate run passed with explicit `PGEN_SOTA_SV_*` trial-shape overrides and logged effective forwarded values.

### 2026-03-01: Made aggregate parse-full promotion target policy-driven
- Root cause:
  - aggregate promotion stage was enabled, but target threshold was effectively implicit script default and not centrally policy-driven in `sota_exit_gate`.
- Fix:
  - added aggregate policy/runtime knobs:
    - `PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO`
    - `PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO`
  - wired validation and forwarding into aggregate stage invocation.
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh` passed,
  - focused aggregate run passed and printed effective:
    - `sv_parse_full_ratio_promotion_target_min_ratio: 20`.

### 2026-03-01: Added structured blocker taxonomy to parse-full ratio promotion reports
- Root cause:
  - promotion `hold` outcomes lacked explicit blocker attribution and required manual log inspection to distinguish ratio debt from non-ratio failures.
- Fix:
  - extended `sv_parse_full_ratio_promotion_gate` report surface:
    - per-trial `blocker_key` + `blocker_detail`,
    - aggregate `blockers` section with breakdown and `primary_non_ratio_blocker`.
  - added blocker classifier signatures for semantic baseline, semantic contract suites, parse-full adapter/report availability, and generic stage failure fallback.
- Validation:
  - default run classifies ratio-only debt as:
    - `parse_full_ratio_threshold_not_met`.
  - forced semantic-closure scenario classifies non-ratio blocker as:
    - `semantic_baseline_validation_failed`.
  - focused aggregate run with informational promotion stage remained green.

### 2026-03-01: Aligned parse-full ratio promotion defaults to aggregate policy profile
- Root cause:
  - promotion trials defaulted to semantic-closure profile (`sv_semantic_file` + semantic closure on), which could fail for semantic reasons unrelated to parse-full ratio ratchet decisions.
- Fix:
  - updated promotion gate defaults to aggregate profile surface:
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE=0`
    - `PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE=sv_file`
- Validation:
  - standalone promotion gate now reports ratio-only debt (`trial_failed`) with zero non-ratio trial failures (`trial_gate_failures=0`),
  - focused aggregate run with promotion stage enabled informationally remains green.

### 2026-03-01: Added deterministic SV parse-full ratio promotion gate + aggregate informational wiring
- Root cause:
  - aggregate parse-full ratio ratcheting had no dedicated promotion-trial contract, forcing ad-hoc manual threshold decisions.
- Fix:
  - added `sv_parse_full_ratio_promotion_gate`:
    - deterministic strict trial matrix with target-threshold parse-full ratio enforcement,
    - recommendation report (`raise_min_parse_full_pass_ratio` vs `hold`) at:
      - `rust/target/sv_parse_full_ratio_promotion_gate/work/systemverilog_parse_full_ratio_promotion_report.json`.
  - wired `sota_exit_gate` policy/runtime controls:
    - `PGEN_SOTA_POLICY_RUN_SV_PARSE_FULL_RATIO_PROMOTION` / `PGEN_SOTA_RUN_SV_PARSE_FULL_RATIO_PROMOTION`
    - `PGEN_SOTA_POLICY_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT` / `PGEN_SOTA_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT`
  - set tracked default to informational-first (`run=1`, `strict=0`).
- Validation:
  - script syntax checks passed (`bash -n` for promotion gate and aggregate gate),
  - standalone promotion gate executed and emitted summary/report artifacts,
  - focused aggregate run executed promotion stage in informational mode with required-check set held stable.

### 2026-03-01: Ratcheted aggregate SV parse-full minimum pass ratio to 15%
- Root cause:
  - aggregate SV parse-full ratio enforcement was active at `10%`; roadmap next step called for controlled threshold ratcheting while preserving deterministic green runs.
- Fix:
  - updated tracked policy default:
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO=15` (was `10`)
  - synced UG/roadmap/development notes.
- Validation:
  - strict semantic-closure run with `min=15` passed:
    - `parse_full_pass_ratio_percent=16`
  - focused aggregate strict run (`sv_stimuli_quality_gate` required) passed and reported:
    - `sv_stimuli_min_parse_full_pass_ratio: 15`

### 2026-03-01: Wired SV parse-full ratio controls into aggregate `sota_exit_gate` policy path
- Root cause:
  - parse-full ratio enforcement existed in `sv_stimuli_quality_gate`, but aggregate policy had no dedicated knobs to forward and require it consistently.
- Fix:
  - extended `sota_exit_gate` with policy/runtime knobs:
    - `PGEN_SOTA_POLICY_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO`
    - `PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO`
    - `PGEN_SOTA_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO`
    - `PGEN_SOTA_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO`
  - added validation and forwarding of these knobs to `sv_stimuli_quality_gate`.
  - updated tracked policy defaults to enforce ratio floor:
    - `enforce=1`, `min=10`.
- Validation:
  - `bash -n rust/scripts/sota_exit_gate.sh` passed.
  - focused aggregate strict run (`sv_stimuli_quality_gate` required) passed with policy-forwarded parse-full ratio enforcement.

### 2026-03-01: Added parse-full quality telemetry + optional strict threshold in SV stimuli gate
- Root cause:
  - semantic-closure parse-full debt lacked an objective contract surface; parse-full remained mostly soft-fail telemetry without threshold controls.
- Fix:
  - promoted `systemverilog_core_v0_contract.json` to `v21` with:
    - `parse_full_quality.enforce_min_pass_ratio`
    - `parse_full_quality.min_pass_ratio`
  - extended `sv_stimuli_quality_gate`:
    - computes `parse_full_pass_ratio_percent`,
    - emits deterministic report `systemverilog_parse_full_quality_report.json`,
    - supports env overrides:
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO`
      - `PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO`
    - strict mode now fails if parse-full is unavailable or ratio is below configured minimum.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - semantic-closure run passed with telemetry (`parse_full_pass_ratio_percent=16`).
  - strict threshold smoke (`enforce=1`, `min=10`) passed.
  - focused aggregate strict run (`sota_exit_gate` with `sv_stimuli_quality_gate` required) passed.

### 2026-03-01: Promoted runtime declaration-before-use in semantic-closure profile with parseability guardrails
- Root cause:
  - promotion readiness and strict aggregate promotion-stage policy were green, but runtime semantic enforcement (`require_declared_identifiers_before_use`) was still disabled in `sv_semantic_file`.
  - direct unguarded enablement produced semantic failures on non-parseable samples.
- Fix:
  - promoted `systemverilog_core_v0_contract.json` to `v20`:
    - `sv_semantic_file.semantic_overrides.require_declared_identifiers_before_use=true`
    - `sv_semantic_file.semantic_overrides.require_declared_identifiers_parseable_only=true`
    - baseline default `semantic_baseline.require_declared_identifiers_parseable_only=false`
  - updated `sv_stimuli_quality_gate` semantic baseline evaluation to consume parse status and skip declaration checks on non-parseable samples when parseable-only guard is enabled.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=6 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` passed with:
    - `semantic_require_declared_identifiers_before_use: 1`
    - `semantic_require_declared_identifiers_parseable_only: 1`
    - `semantic_baseline_passes: 12/12`
  - `make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate` remained green (`enable_runtime_declared_identifiers`).

### 2026-03-01: Promoted declared-shadow promotion stage to required strict policy
- Root cause:
  - promotion-trial evidence had converged (`enable_runtime_declared_identifiers`) but aggregate policy still ran the stage informational-only.
- Fix:
  - updated `rust/config/sota_exit_policy.env`:
    - `PGEN_SOTA_POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT=1` (was `0`)
  - this makes aggregate `sota_exit_gate` execute `sv_declared_shadow_promotion_gate` in strict mode by default.
- Validation:
  - focused aggregate run passed with strict promotion stage enabled:
    - `PGEN_SOTA_REQUIRED_CHECKS=differential_baseline_contract ... PGEN_SOTA_RUN_SV_DECLARED_SHADOW_PROMOTION=1 PGEN_SOTA_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT=1 ... rust/scripts/sota_exit_gate.sh`

### 2026-03-01: Stabilized declared-shadow promotion defaults (parseable-yield profile)
- Root cause:
  - parseability-scoped strict-shadow trials were correct, but low default per-trial sample counts still produced under-sampled outcomes (`checked=0`) on baseline seed paths.
- Fix:
  - updated `rust/scripts/sv_declared_shadow_promotion_gate.sh` defaults:
    - `PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT=6` (was `2`)
    - new `PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE=sv_file`
  - strict trial runner now forwards selected promotion stimuli mode:
    - `PGEN_SV_STIMULI_QUALITY_MODE=$PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE`
  - promotion report now records `promotion_stimuli_mode`.
- Validation:
  - `bash -n rust/scripts/sv_declared_shadow_promotion_gate.sh` passed.
  - `make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate` passed.
  - baseline deterministic recommendation now:
    - `recommendation=enable_runtime_declared_identifiers`
    - `eligible_for_runtime_enforcement=1`
    - `totals_checked=5`
    - `totals_failed=0`

### 2026-03-01: Scoped strict-shadow trials to parseable samples (promotion burn-down)
- Root cause:
  - strict-shadow failures were dominated by lexically noisy, non-parseable generated samples, which blurred semantic-vs-syntax debt and created false-positive undeclared-identifier noise.
- Fix:
  - added `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY=0|1` in `sv_stimuli_quality_gate`.
  - when enabled, shadow checks now run only for `parse_full=pass` samples; unparseable samples are tracked as `skip_unparseable`.
  - strict-shadow now fails explicitly if parseable-only filtering yields zero checked samples.
  - updated promotion gate defaults:
    - `parse_full_mode=auto`
    - `min_checked=2`
    - always sets `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY=1` for trial runs.
- Validation:
  - shell syntax checks passed for both gate scripts.
  - promotion trial on baseline seed path (`12001`) now reports:
    - `recommendation=hold`
    - `checked=0`
    - `skipped_unparseable=2`
  - blocker is now explicit parseability debt, not lexical undeclared-id noise.

### 2026-02-28: Added declared-shadow promotion trial gate and aggregate informational wiring
- Root cause:
  - strict-shadow telemetry existed, but promotion readiness for enabling runtime `require_declared_identifiers_before_use` lacked a dedicated deterministic decision/report surface.
- Fix:
  - added executable `rust/scripts/sv_declared_shadow_promotion_gate.sh`:
    - runs strict-shadow trial matrix (`semantic_closure_mode=1`, `declared_shadow_mode=1`),
    - emits deterministic recommendation report:
      - `rust/target/sv_declared_shadow_promotion_gate/work/systemverilog_declared_identifier_promotion_report.json`.
  - added `make sv_declared_shadow_promotion_gate` target and help entry.
  - wired stage into aggregate policy as informational-first:
    - `PGEN_SOTA_POLICY_RUN_SV_DECLARED_SHADOW_PROMOTION=1`
    - `PGEN_SOTA_POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT=0`
  - extended `sota_exit_gate` with matching runtime knobs and stage execution.
- Validation:
  - `bash -n rust/scripts/sv_declared_shadow_promotion_gate.sh` passed.
  - `bash -n rust/scripts/sota_exit_gate.sh` passed.
  - promotion gate smoke (`TRIALS=1`, `COUNT=1`, `PARSE_FULL_MODE=0`) passed and emitted recommendation.
  - lightweight aggregate run with only `differential_baseline_contract` + new informational promotion stage passed.
  - baseline seed evidence (`12001`) currently recommends `hold` (`1/2` strict shadow failures), so runtime declared-before-use promotion remains blocked.

### 2026-02-28: Closed Phase R user-facing AST workflow documentation
- Root cause:
  - AST dump implementation/gates were complete, but users still lacked a single operational playbook for how to apply these features during SV/VHDL/regex onboarding and debugging.
- Fix:
  - expanded `PGEN_USER_GUIDE.md` with `AST Debug Playbooks (SV/VHDL/Regex)` covering:
    - generator-input AST flow (`--dump-gen-ast`),
    - parser-returned AST flow (`parseability_probe --parse-dump-ast*`) where adapter is available,
    - embedding API in-memory AST dump usage (`parse_systemverilog_*_ast_dump`),
    - regex onboarding via deterministic stimuli/coverage/gap loop.
  - updated `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`:
    - marked `Document AST dump workflows in user-facing docs` as complete.
- Validation:
  - manual contract consistency pass across guide examples and implemented CLI/API surfaces.
  - no Rust/source changes in this task.

### 2026-02-28: Closed Phase R embedding-API parser AST dump surface
- Root cause:
  - parser-returned AST dump contract was complete for CLI (`parseability_probe`) but missing on embedding API entry points, leaving host integrations without in-memory parser AST dump access.
- Fix:
  - extended `rust/src/embedding_api.rs` with stable AST dump types and APIs:
    - `AstDumpOptions`, `AstDumpPayload`, `GrammarAstDumpOutcome`, `NamedGrammarAstDumpOutcome`,
    - typed APIs: `parse_grammar_profile_ast_dump*`, `parse_systemverilog_*_ast_dump*`, `parse_vhdl_1076_2019_ast_dump*`,
    - named API: `parse_grammar_profile_ast_dump_named*`.
  - added deterministic serializer/truncation contract in embedding path:
    - recursive canonical JSON key-order normalization,
    - compact/pretty control,
    - bounded output with deterministic truncation envelope (`kind=pgen_ast_dump_truncation`, `dump_kind=parser_return_ast`),
    - invalid AST bound handling via `E_INVALID_LIMITS`.
  - updated docs/roadmap:
    - `rust/docs/EMBEDDING_API_CONTRACT.md`
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` (Phase R parser-returned dump item marked complete).
- Validation:
  - `cd rust && cargo test --lib embedding_api` passed.
  - `cd rust && cargo test --features generated_parsers --lib embedding_api` passed.

### 2026-02-28: Closed Phase R gate-level AST dump validation with dedicated contract gate
- Root cause:
  - both AST dump surfaces had deterministic/bounded serialization contracts, but there was no executable gate enforcing replay determinism, truncation behavior, and negative-path write failures.
- Fix:
  - added `rust/scripts/ast_dump_contract_gate.sh` and Make target `ast_dump_contract_gate`.
  - gate verifies:
    - generation-input dump determinism + truncation envelope + negative-path failure,
    - parser-returned dump determinism + truncation envelope + negative-path failure.
  - updated `rust/src/main.rs` to add explicit generation dump write-failure context (`failed to write generation-input AST JSON ...`) for reliable negative-path assertions.
  - synced docs/roadmap:
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- Validation:
  - `make -C rust SHELL=/bin/bash ast_dump_contract_gate` passed.
  - `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` passed.

### 2026-02-28: Extended Phase R dump-format/safety contract to parser-returned AST dumps
- Root cause:
  - generation-input AST dump already had deterministic canonicalization + bounded truncation safeguards, but parser-returned AST dump (`parseability_probe`) still lacked equivalent contract behavior.
- Fix:
  - updated `rust/src/bin/parseability_probe.rs`:
    - added optional dump-tail flag `--max-bytes <N>`,
    - added env fallback `PGEN_PARSE_DUMP_AST_MAX_BYTES`,
    - added recursive JSON key canonicalization before parser AST dump emission,
    - added bounded-size writer with deterministic truncation envelope (`kind=pgen_ast_dump_truncation`, `dump_kind=parser_return_ast`) and explicit too-small-bound failure.
  - added unit tests for tail parsing, canonicalization, and truncation envelope emission.
  - synchronized docs and roadmap:
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml --bin parseability_probe --features generated_parsers` passed.
  - CLI smoke with oversized parser AST and `--max-bytes 256` produced truncation envelope (`kind=pgen_ast_dump_truncation`).

### 2026-02-28: Added Phase R generation-input AST dump safety contract baseline
- Root cause:
  - generation-input AST dump existed but lacked bounded-size safeguards and deterministic key-order normalization guarantees.
- Fix:
  - updated `rust/src/main.rs`:
    - added `--dump-gen-ast-max-bytes` with env fallback `PGEN_DUMP_GEN_AST_MAX_BYTES`,
    - added recursive JSON key canonicalization before dump encoding,
    - added bounded dump writer that emits deterministic truncation diagnostics JSON envelope (`kind=pgen_ast_dump_truncation`) when payload exceeds bound.
  - added regression coverage in `rust/src/main.rs` for:
    - recursive canonicalization behavior,
    - truncation diagnostics emission.
  - updated docs/roadmap:
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml --bin ast_pipeline` passed.
  - bounded dump smoke confirmed `kind=pgen_ast_dump_truncation` when limit is exceeded.

### 2026-02-28: Added probe-preflight hardening for SV preprocessor differential runner availability
- Root cause:
  - even after adding a project-level reference runner, environments without trusted backends (`iverilog`/`verilator`) still generated per-sample mismatches in `DIFF_MODE=auto`, reducing taxonomy signal quality.
- Fix:
  - extended `rust/scripts/sv_preprocessor_reference_runner.sh` with `--probe` availability mode,
  - updated `rust/scripts/sv_preprocessor_quality_gate.sh` to:
    - detect probe support,
    - preflight backend availability before differential sample classification,
    - downgrade to `unsupported_reference_runner` in auto mode on probe failure,
    - fail fast in strict mode on probe failure with explicit probe-log path.
  - synchronized docs/roadmap:
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- Validation:
  - `bash -n rust/scripts/sv_preprocessor_reference_runner.sh` passed.
  - `bash -n rust/scripts/sv_preprocessor_quality_gate.sh` passed.
  - reduced auto mode run passed with `diff_mode_effective=unsupported_reference_runner` when backend unavailable.
  - reduced strict mode run failed early as expected with probe failure message.

### 2026-02-28: Standardized trusted-reference runner adapter for SV preprocessor differential gate
- Root cause:
  - Phase Q differential taxonomy in `sv_preprocessor_quality_gate` was implemented, but there was no canonical project runner adapter for trusted-reference preprocessors.
- Fix:
  - added executable runner shim:
    - `rust/scripts/sv_preprocessor_reference_runner.sh`
    - contract-compatible args (`$1` input, `$2` preprocessed output, `$3` diagnostics JSON),
    - backend routing:
      - `PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND=auto|iverilog|verilator` (`auto` prefers `iverilog`, then `verilator`),
    - include/define forwarding:
      - `PGEN_SV_PREPROCESSOR_REFERENCE_INCLUDE_DIRS`
      - `PGEN_SV_PREPROCESSOR_REFERENCE_DEFINES`
    - deterministic diagnostics emission (always JSON array).
  - synchronized docs/roadmap:
    - `PGEN_USER_GUIDE.md`
    - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- Validation:
  - `bash -n rust/scripts/sv_preprocessor_reference_runner.sh` passed.
  - failure-path smoke in local environment without `iverilog`/`verilator`:
    - runner exited non-zero,
    - diagnostics JSON array emitted with `error` severity entry.

### 2026-02-28: Added declared-identifier shadow burn-down telemetry for Phase P semantic-promotion
- Root cause:
  - deterministic suites are in place for semantic checks, but runtime `require_declared_identifiers_before_use` still remains disabled in semantic-closure mode due residual lexical-edge false-positive risk.
- Fix:
  - promoted contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v19`
    - added `semantic_promotion.declared_identifier_shadow_enabled`
    - added `semantic_promotion.declared_identifier_shadow_strict`
  - extended gate:
    - `rust/scripts/sv_stimuli_quality_gate.sh`
    - new override:
      - `PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE=auto|0|1`
    - per-sample shadow checks now run for declared-identifier validation when runtime enforcement is off,
    - deterministic report output:
      - `systemverilog_declared_identifier_shadow_report.json`.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Added deterministic SV context-legality contract suite (Phase P semantic closure)
- Root cause:
  - semantic-closure deterministic suites covered declared-before-use/width/port-binding/package-qualification checks, but lacked fixed-corpus coverage for context-legality behavior.
- Fix:
  - added suite corpus:
    - `rust/test_data/grammar_quality/systemverilog_context_legality_contract_cases.json`
  - promoted contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v18`
    - added:
      - `semantic_contracts.context_legality_suite_path`
      - `semantic_contracts.enforce_context_legality_suite`
  - extended `rust/scripts/sv_stimuli_quality_gate.sh`:
    - new stage: `context_legality_contract_suite`,
    - new env overrides:
      - `PGEN_SV_STIMULI_QUALITY_CONTEXT_LEGALITY_SUITE`
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_CONTEXT_LEGALITY_SUITE`,
    - summary counters for suite status/total/passed/failed.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_context_legality_contract_cases.json` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Added deterministic SV package-qualification contract suite (Phase P semantic closure)
- Root cause:
  - semantic-closure deterministic suites covered declared-before-use/width/port-binding checks, but lacked fixed-corpus coverage for package qualification resolution behavior.
- Fix:
  - added suite corpus:
    - `rust/test_data/grammar_quality/systemverilog_package_qualification_contract_cases.json`
  - promoted contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v17`
    - added:
      - `semantic_contracts.package_qualification_suite_path`
      - `semantic_contracts.enforce_package_qualification_suite`
  - extended `rust/scripts/sv_stimuli_quality_gate.sh`:
    - new stage: `package_qualification_contract_suite`,
    - new env overrides:
      - `PGEN_SV_STIMULI_QUALITY_PACKAGE_QUAL_SUITE`
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_PACKAGE_QUAL_SUITE`,
    - summary counters for suite status/total/passed/failed.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_package_qualification_contract_cases.json` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Added deterministic SV port-binding legality contract suite (Phase P semantic closure)
- Root cause:
  - semantic-closure deterministic suites covered declared-before-use and width compatibility, but lacked a fixed pass/fail corpus for named-port legality behavior.
- Fix:
  - added suite corpus:
    - `rust/test_data/grammar_quality/systemverilog_port_binding_legality_contract_cases.json`
  - promoted contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v16`
    - added:
      - `semantic_contracts.port_binding_legality_suite_path`
      - `semantic_contracts.enforce_port_binding_legality_suite`
  - extended `rust/scripts/sv_stimuli_quality_gate.sh`:
    - new stage: `port_binding_legality_contract_suite`,
    - new env overrides:
      - `PGEN_SV_STIMULI_QUALITY_PORT_BINDING_SUITE`
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_PORT_BINDING_SUITE`,
    - summary counters for suite status/total/passed/failed.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_port_binding_legality_contract_cases.json` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Added contractized SV stimuli performance/memory-proxy budget stage (Phase P)
- Root cause:
  - Phase P differential/integration hardening required enforceable performance and memory-proxy guardrails, but `sv_stimuli_quality_gate` had no deterministic budget contract/report.
- Fix:
  - promoted `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` to `v15` with `performance_budgets`:
    - `enforce`
    - `max_generate_ms_per_sample`
    - `max_preprocess_ms_per_sample`
    - `max_parse_full_ms_per_sample`
    - `max_sample_bytes`
    - `max_preprocessed_bytes`
  - extended `rust/scripts/sv_stimuli_quality_gate.sh`:
    - mode control: `PGEN_SV_STIMULI_PERF_BUDGET_MODE=auto|0|1`,
    - per-sample timing checks for generation/preprocess/parse_full (when active),
    - per-sample size checks for generated/preprocessed artifacts,
    - deterministic report output:
      - `rust/target/sv_stimuli_quality_gate/work/systemverilog_performance_report.json`.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_PERF_BUDGET_MODE=1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Added trusted-reference differential taxonomy stage to `sv_stimuli_quality_gate` (Phase P)
- Root cause:
  - Phase P required mismatch taxonomy against trusted references for Nexsim parser hardening, but `sv_stimuli_quality_gate` had no executable differential stage.
- Fix:
  - added differential controls in `rust/scripts/sv_stimuli_quality_gate.sh`:
    - `PGEN_SV_STIMULI_DIFF_MODE=auto|0|1`
    - `PGEN_SV_STIMULI_DIFF_MAX_SAMPLES`
    - `PGEN_SV_STIMULI_REFERENCE_RUNNER`
  - added trusted-reference differential execution over preprocessed samples using:
    - Rust parseability via `parseability_probe --parse systemverilog`,
    - reference parseability via runner interface (`$1 input`, `$2 ast_json`, `$3 diagnostics_json`).
  - added deterministic taxonomy report artifact:
    - `rust/target/sv_stimuli_quality_gate/work/systemverilog_differential_report.json`
  - added strict-mode behavior:
    - fail on missing prerequisites,
    - fail on asymmetric mismatch categories.
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 PGEN_SV_STIMULI_DIFF_MODE=0 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_DIFF_MODE=auto PGEN_SV_STIMULI_REFERENCE_RUNNER=<shim> PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=400 bash rust/scripts/sv_stimuli_quality_gate.sh` passed.

### 2026-02-28: Enabled executable parseability validation for `systemverilog_preprocessor` in Phase Q gate
- Root cause:
  - `sv_preprocessor_quality_gate` parseability stage ran in auto mode but fell back to `unsupported_adapter` because there was no dynamic generated-parser adapter path for grammar `systemverilog_preprocessor`.
- Fix:
  - wired dynamic preprocessor parser support:
    - `rust/build.rs`: new cfg/env path support (`has_generated_systemverilog_preprocessor_parser`, `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`),
    - `rust/src/lib.rs`: cfg-gated `generated_parsers::systemverilog_preprocessor`,
    - `rust/src/parser_registry.rs`: parseability + AST-JSON adapters for grammar name `systemverilog_preprocessor`.
  - hardened `rust/scripts/sv_preprocessor_quality_gate.sh`:
    - gate now generates `systemverilog_preprocessor_parser.rs` into gate workdir,
    - rebuilds `ast_pipeline` with `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` so `--validate-parseability` is active in the same run.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml parser_registry --features generated_parsers` passed.
  - reduced-cost gate run passed:
    - `PGEN_SV_PREPROCESSOR_QUALITY_COUNT=1 PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS=1 PGEN_SV_PREPROCESSOR_DIFF_MODE=0 PGEN_SV_PREPROCESSOR_QUALITY_TARGET_MAX_ATTEMPTS=400 PGEN_SV_PREPROCESSOR_QUALITY_GAP_THRESHOLD=1 bash rust/scripts/sv_preprocessor_quality_gate.sh`
  - gate summary now reports `parseability_mode_effective: enabled`.

### 2026-02-28: Phase P semantic-closure deterministic width-compatibility contract suite
- Root cause:
  - semantic-closure had deterministic declared-identifier contract proofs, but width-compatibility behavior lacked fixed pass/fail contract corpus enforcement.
- Fix:
  - extended `rust/scripts/sv_stimuli_quality_gate.sh` with `width_compatibility_contract_suite` stage and summary counters.
  - added contract/env controls:
    - contract:
      - `semantic_contracts.width_compatibility_suite_path`
      - `semantic_contracts.enforce_width_compatibility_suite`
    - env:
      - `PGEN_SV_STIMULI_QUALITY_WIDTH_COMPAT_SUITE`
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_WIDTH_COMPAT_SUITE`
  - added deterministic corpus:
    - `rust/test_data/grammar_quality/systemverilog_width_compatibility_contract_cases.json`
  - promoted core contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` (`v13 -> v14`).
  - updated roadmap progress in Phase P semantic-closure item.
- Validation:
  - `jq empty rust/test_data/grammar_quality/systemverilog_width_compatibility_contract_cases.json`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 bash rust/scripts/sv_stimuli_quality_gate.sh`
  - result: pass.

### 2026-02-28: Generator clippy-deny cleanup for generated parser flows
- Root cause:
  - strict clippy over generated parser targets failed due generator-emitted constant-expression patterns:
    - `false && ...` (`overly_complex_bool_expr`)
    - `0usize == 0usize` (`eq_op`)
    - `0usize > best_branch_index` (`absurd_extreme_comparisons`)
  - fallback return-transform emission also caused syntax/move fragility in non-bootstrap EBNF generation paths.
- Fix:
  - `rust/src/ast_pipeline/ast_return_transform.rs`
    - split positional extraction codegen at generation-time (`index==0` vs `>0`),
    - removed constant guard emission and replaced with direct index-0 handling.
  - `rust/src/ast_pipeline/ast_based_generator.rs`
    - quantifier guard emission now token-specialized (`stop_at_rule_boundary_on_break` / `..._on_error`) instead of runtime `false && ...`,
    - OR tie-break compares runtime `current_branch_index` instead of constant literal branch indices,
    - fixed fallback transform emission correctness and moved-value safety (`result.clone()`),
    - updated generated test logger path to `crate::ast_pipeline::NoOpLogger` for bin-target compatibility.
  - Regeneration path:
    - bootstrap-only: `make -C rust return_annotation_parser semantic_annotation_parser`
    - non-bootstrap EBNF: `cargo run --features generated_parsers --bin ast_pipeline -- ../generated/ebnf.json --generate-parser --eliminate-left-recursion -o ../generated/ebnf.rs`
- Validation:
  - `cargo clippy --manifest-path rust/Cargo.toml --all-targets --features generated_parsers,ebnf_dual_run`
  - result: pass (`EXIT:0`).

### 2026-02-28: Workflow hardening - clippy auto-flow for Rust/generated Rust changes
- Root cause:
  - clippy was run ad hoc and not embedded as a mandatory workflow step after Rust or generated parser changes.
- Fix:
  - added executable workflow script:
    - `rust/scripts/clippy_on_rust_change.sh`
    - detects Rust/generated Rust changes from working tree/index/untracked set and runs:
      - strict source clippy: `cargo clippy --all-targets`
      - generated integration clippy: `cargo clippy --all-targets --features generated_parsers,ebnf_dual_run`
    - generated stage is report-mode by default (current generated parser debt), strict when `PGEN_CLIPPY_GENERATED_STRICT=1`.
  - added make target:
    - `make -C rust clippy_on_rust_change`
  - updated commit workflow contract:
    - `COMMIT.md` now requires running this clippy flow whenever Rust/generated Rust files are changed.
- Validation:
  - `PGEN_CLIPPY_FORCE=1 make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`
  - source clippy path passes; generated clippy path runs and reports existing generated-parser clippy debt in non-strict mode.
- Status:
  - clippy execution is now part of the standard task-completion workflow for Rust code changes.

### 2026-02-28: Phase P deterministic declared-identifier semantic contract suite
- Root cause:
  - declaration-before-use checker improvements existed, but there was no deterministic fixed corpus proving expected pass/fail behavior independently of random stimuli variability.
- Fix:
  - added deterministic suite:
    - `rust/test_data/grammar_quality/systemverilog_declared_identifier_contract_cases.json`
  - bumped core contract:
    - `rust/test_data/grammar_quality/systemverilog_core_v0_contract.json` (`v12 -> v13`)
    - added `semantic_contracts.declared_identifier_suite_path`
    - added `semantic_contracts.enforce_declared_identifier_suite`
  - wired `sv_stimuli_quality_gate` pre-stage:
    - `declared_identifier_contract_suite`
    - env overrides:
      - `PGEN_SV_STIMULI_QUALITY_DECLARED_IDENTIFIER_SUITE`
      - `PGEN_SV_STIMULI_QUALITY_ENFORCE_DECLARED_IDENTIFIER_SUITE`
    - summary counters:
      - `declared_identifier_suite_status/total/passed/failed`
  - fixed `foreach` iterator declaration extraction:
    - `foreach (arr[idx])` now correctly declares `idx` before use checks.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `jq empty rust/test_data/grammar_quality/systemverilog_declared_identifier_contract_cases.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - deterministic suite passes `12/12`; both baseline and semantic-closure gate modes remain green.

### 2026-02-27: Phase P declared-check hardening - structured use-site extraction
- Root cause:
  - declaration-before-use still produced lexical-noise false positives when scanning all identifiers globally on randomized semantic-closure stimuli.
- Fix:
  - reworked `check_declared_identifiers_before_use` in `rust/scripts/sv_stimuli_quality_gate.sh` to structured use-site scanning:
    - assignment LHS/RHS,
    - condition expressions,
    - event controls,
    - named-port actual expressions.
  - retained lexical sanitization/context skips (strings/directives/timeunit-timeprecision/member-namespace-macro path filtering).
  - kept `sv_semantic_file` contract posture:
    - `require_declared_identifiers_before_use=false`
    - `require_width_compatibility_simple=true`
- Validation:
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - semantic-closure mode remains stable with stronger validator internals; declaration-before-use remains intentionally deferred for final lexical-edge burn-down.

### 2026-02-27: Phase P semantic-validator hardening for `sv_semantic_file`
- Root cause:
  - enabling both declaration-before-use and width checks in semantic-closure mode exposed lexical false positives (for example `timeunit` tokenization artifacts) on random stimuli.
- Fix:
  - hardened `check_declared_identifiers_before_use` in `rust/scripts/sv_stimuli_quality_gate.sh`:
    - strips quoted strings and `timeunit/timeprecision` lines,
    - ignores member/namespace/macro contexts,
    - expands declaration context extraction and keyword coverage.
  - hardened `check_width_compatibility_simple`:
    - added packed-width extraction for `logic|reg|wire|bit`,
    - supports indexed LHS assignment forms.
  - tightened `sv_semantic_file` policy in `systemverilog_core_v0_contract.json`:
    - enabled `require_width_compatibility_simple=true`,
    - kept `require_declared_identifiers_before_use=false` until lexical-noise debt is further reduced.
  - synced roadmap/UG/docs notes for current semantic-closure policy posture.
- Validation:
  - `bash -n rust/scripts/sv_stimuli_quality_gate.sh`
  - `jq empty rust/test_data/grammar_quality/systemverilog_core_v0_contract.json`
  - `PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=1 PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
- Status:
  - semantic-closure mode now carries stronger width checks with stable pass behavior; declaration-before-use remains intentionally deferred pending additional robustness work.

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

### 2026-02-28: Added roadmap Phase R for AST dump observability planning
- Root cause:
  - AST dump debug needs were identified for two surfaces (generator-input AST and generated-parser returned AST) but were not tracked as explicit roadmap deliverables.
- Fix:
  - Added `Phase R (AST Observability and Debug Artifacts)` in `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` with planned work items for:
    - generator-input AST dump (`gen_ast.json`) CLI support,
    - generated-parser returned-AST dump (`<grammar>_ast.json`) runtime/API support,
    - deterministic dump formatting/safety contracts,
    - gate-level validation and UG documentation tasks.
  - Synced planning entry into `CHANGES.md` and `DEVELOPMENT_NOTES.md`.
- Validation:
  - docs-only planning increment; no runtime behavior changed.

### 2026-02-28: Implemented Phase R item 1 (`--dump-gen-ast`) in `ast_pipeline`
- Root cause:
  - There was no executable CLI feature to dump the normalized AST consumed by generation paths (`--generate-parser`, `--generate-stimuli`, `--generate-stimuli-module`).
- Fix:
  - Added CLI options in `rust/src/main.rs`:
    - `--dump-gen-ast [PATH]` with deterministic default `gen_ast.json`,
    - `--dump-gen-ast-pretty` for pretty JSON mode.
  - Added mode guard:
    - dump option is valid only for generation modes.
  - Added shared dump helper:
    - serializes `grammar_name`, `rule_order`, `grammar_tree`, and `annotations`.
  - Wired dump helper into all generation paths after grammar load.
  - Added serialization derives in `rust/src/ast_pipeline/mod.rs` for AST/annotation payload types needed by dump output.
  - Added tests in `rust/src/main.rs` covering dump artifact content and pretty formatting.
  - Updated `PGEN_USER_GUIDE.md` with a dedicated generation-input AST dump section and examples.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml --bin ast_pipeline` passed.
  - `cargo clippy --manifest-path rust/Cargo.toml --all-targets --features generated_parsers,ebnf_dual_run` passed (no clippy errors).
  - smoke run confirmed artifact output:
    - `cargo run --manifest-path rust/Cargo.toml --bin ast_pipeline -- generated/json.json --generate-stimuli --count 1 --dump-gen-ast /tmp/pgen_gen_ast.json --output /tmp/pgen_stimuli.txt`.

### 2026-02-28: Implemented parser-returned AST dump CLI with grammar-based JSON defaults
- Root cause:
  - parser-returned AST dump naming used a generic log-style name and did not encode grammar identity by default.
- Fix:
  - updated parser dump naming contract to `<grammar>_ast.json`,
  - added parseability CLI commands:
    - `--parse-dump-ast`
    - `--parse-dump-ast-pretty`
  - added `parser_registry::parse_sample_ast_json(...)` to expose serialized parser-returned AST JSON,
  - added `Serialize` derives for parse tree payloads (`ParseContent`, `ParseNode`) to enable JSON dump output.
- Validation:
  - `cargo test --manifest-path rust/Cargo.toml parser_registry --features generated_parsers,ebnf_dual_run` passed.
  - `cargo test --manifest-path rust/Cargo.toml --bin parseability_probe --features generated_parsers,ebnf_dual_run` passed.

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
1. Continue Phase P semantic-closure implementation for SV:
   - runtime declaration-before-use is enabled and parse-full quality thresholding is aggregate-policy enforced (`enforce=1`, `min=100`),
   - promotion evidence at target `100` now includes denser deterministic shape (`trial_passed=4/4` at `count=8`, observed ratio `100/100/100`) and contract-default `8/8` closed-loop stress, so next work should broaden seed-space/corpus families while preserving semantic-suite closure.
2. Add annotation-driven SV stimuli steering:
   - initial baseline + rule-level expansion are in place, parseable-subset mode (`sv_parseable_file`) is contractized with `100%` parse-full, and `sv_file` deterministic run is now also at `100%` for current trial shape,
   - next increment should broaden trial shapes/corpus stress (counts/seeds/contracts) while preserving `sv_file` parse-full stability and semantic-suite closure.
3. Expand contractized SV/VHDL corpora:
   - SV preprocess-heavy deterministic semantic suite increment is done (`version: 2` across enforced SV semantic suites),
   - next corpus increment should target VHDL deterministic semantic/parseability families and additional SV parse-full-improving families.
4. Promote VHDL aggregate mode from informational to strict-required when stability criteria are met:
   - keep `PGEN_SOTA_POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT=0` until deterministic pass rate is proven across broader corpus.
5. Continue Rust-native EBNF migration hardening:
   - preserve parity/dual-run contracts while reducing Perl frontend dependence.
6. Keep roadmap + UG + memory synced after every gate/contract increment.

## Known Gaps / Risks
- Pipeline is still hybrid (`ebnf_to_json.pl` remains active in core/gate flows).
- Rust EBNF frontend exists and is validated via dual-run, but is not full replacement yet.
- Semantic-annotation leverage in SV/VHDL stimuli generation is still partial; mode-level policy exists but full directive-driven steering is not closed yet.
- Declared-before-use runtime semantic enforcement is active in `sv_semantic_file` with parseability guardrails; parse-full debt is now measured (`parse_full_pass_ratio_percent`) but still significant in semantic-closure mode.
- Aggregate VHDL stimuli gate is currently informational-first; strict promotion is pending additional stability evidence.
- SV preprocessor trusted-reference differential path still depends on host availability of external backends (`iverilog`/`verilator`), but offline curated differential gate now provides deterministic no-external baseline evidence.
- Phase R is fully closed: implementation + gate-level validation + embedding API + end-user workflow playbooks are now complete.

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
- AST dump contract gate:
  - `make -C rust SHELL=/bin/bash ast_dump_contract_gate`
- Declared-shadow promotion trial gate:
  - `make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate`
- SV preprocessor strict differential example:
  - `PGEN_SV_PREPROCESSOR_DIFF_MODE=1 PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER=$PWD/rust/scripts/sv_preprocessor_reference_runner.sh PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND=auto make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- Aggregate gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
