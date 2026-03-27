#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
STATE_DIR="${PGEN_CI_WORKFLOW_LOCAL_STATE_DIR:-$RUST_DIR/target/ci_workflow_local_gate}"
mkdir -p "$STATE_DIR"
RUN_DIR="$(mktemp -d "$STATE_DIR/run.XXXXXX")"
EXPORT_DIR="$RUN_DIR/export"
LOG_DIR="$RUN_DIR/logs"
SUMMARY_FILE="$RUN_DIR/summary.txt"
FILTER_RAW="${PGEN_CI_WORKFLOW_LOCAL_FILTER:-}"
CARGO_OFFLINE_RAW="${PGEN_CI_WORKFLOW_LOCAL_CARGO_OFFLINE:-true}"

mkdir -p "$EXPORT_DIR" "$LOG_DIR"

fail() {
  echo "error: $*" >&2
  exit 1
}

note() {
  echo "$*" | tee -a "$SUMMARY_FILE"
}

require_tool() {
  local tool="$1"
  command -v "$tool" >/dev/null 2>&1 || fail "required tool not found on PATH: $tool"
}

is_selected() {
  local workflow_name="$1"
  local item
  if [[ -z "$FILTER_RAW" ]]; then
    return 0
  fi
  IFS=',' read -r -a items <<<"$FILTER_RAW"
  for item in "${items[@]}"; do
    if [[ "$workflow_name" == "$item" ]]; then
      return 0
    fi
  done
  return 1
}

assert_tracked() {
  local repo_rel="$1"
  (cd "$ROOT_DIR" && git ls-files --error-unmatch -- "$repo_rel" >/dev/null 2>&1) || \
    fail "required tracked file missing from git index: $repo_rel"
}

assert_workflow_contains() {
  local workflow_file="$1"
  local expected="$2"
  grep -F -- "$expected" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || \
    fail "workflow content drift detected in $workflow_file: expected '$expected'"
}

assert_workflow_not_contains() {
  local workflow_file="$1"
  local forbidden="$2"
  if grep -F -- "$forbidden" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1; then
    fail "unexpected workflow content in $workflow_file: found '$forbidden'"
  fi
}

assert_file_contains() {
  local repo_file="$1"
  local expected="$2"
  grep -F -- "$expected" "$ROOT_DIR/$repo_file" >/dev/null 2>&1 || \
    fail "file content drift detected in $repo_file: expected '$expected'"
}

assert_file_not_contains() {
  local repo_file="$1"
  local forbidden="$2"
  if grep -F -- "$forbidden" "$ROOT_DIR/$repo_file" >/dev/null 2>&1; then
    fail "unexpected file content in $repo_file: found '$forbidden'"
  fi
}

copy_tracked_worktree() {
  local repo_rel
  note "exporting tracked working tree into $EXPORT_DIR"
  while IFS= read -r -d '' repo_rel; do
    mkdir -p "$EXPORT_DIR/$(dirname "$repo_rel")"
    cp -a "$ROOT_DIR/$repo_rel" "$EXPORT_DIR/$repo_rel"
  done < <(cd "$ROOT_DIR" && git ls-files -z)
}

audit_static_include_paths() {
  note "auditing include!(...) literals"
  if (cd "$ROOT_DIR" && rg -n 'include!\\(\"/' rust/src rust/src/bin -g '*.rs' >/dev/null 2>&1); then
    fail "absolute include!(...) literal found in rust/src or rust/src/bin"
  fi

  assert_tracked "generated/ebnf.rs"
  assert_tracked "generated/return_annotation_parser.rs"
  assert_tracked "generated/semantic_annotation_parser.rs"
}

audit_workflow_surface() {
  local workflow_file
  note "auditing tracked workflow surface"
  for workflow_file in \
    .github/workflows/annotation-contract-gate.yml \
    .github/workflows/annotation-nonbootstrap-e2e-gate.yml \
    .github/workflows/branch-protection-contract-gate.yml \
    .github/workflows/differential-regression-gate.yml \
    .github/workflows/ebnf-frontend-dual-run-diff.yml \
    .github/workflows/fixed-point-gate.yml \
    .github/workflows/performance-gate.yml \
    .github/workflows/sota-exit-gate.yml; do
    assert_tracked "$workflow_file"
  done

  assert_tracked "rust/config/branch_protection_policy.json"
  assert_tracked "rust/config/sota_exit_policy.env"
  assert_tracked "rust/scripts/annotation_nonbootstrap_e2e_gate.sh"
  assert_tracked "rust/scripts/branch_protection_contract_gate.sh"
  assert_tracked "rust/scripts/ebnf_frontend_dual_run_diff_gate.sh"
  assert_tracked "rust/scripts/fixed_point_bootstrap_gate.sh"
  assert_tracked "rust/scripts/performance_gate.sh"
  assert_tracked "rust/scripts/sota_exit_gate.sh"

  assert_workflow_contains \
    ".github/workflows/ebnf-frontend-dual-run-diff.yml" \
    "Verify Perl runtime for Perl-vs-Rust dual-run"
  assert_workflow_contains \
    ".github/workflows/sota-exit-gate.yml" \
    "Verify Perl runtime for SOTA dual-run surfaces"

  for workflow_file in \
    .github/workflows/annotation-contract-gate.yml \
    .github/workflows/annotation-nonbootstrap-e2e-gate.yml \
    .github/workflows/branch-protection-contract-gate.yml \
    .github/workflows/differential-regression-gate.yml \
    .github/workflows/fixed-point-gate.yml \
    .github/workflows/performance-gate.yml; do
    assert_workflow_not_contains "$workflow_file" "Verify Perl runtime"
  done
}

audit_ebnf_frontend_conversion_surface() {
  local repo_file
  note "auditing ebnf_to_json conversion surface"

  for repo_file in \
    rust/Makefile \
    rust/scripts/annotation_nonbootstrap_e2e_gate.sh \
    rust/scripts/fixed_point_bootstrap_gate.sh \
    rust/scripts/hdl_frontend_readiness_gate.sh \
    rust/scripts/stimuli_module_parity_gate.sh \
    rust/scripts/sv_external_corpus_triage_gate.sh \
    rust/scripts/sv_preprocessor_quality_gate.sh \
    rust/scripts/sv_stimuli_quality_gate.sh \
    rust/scripts/sv_syntax_closure_gate.sh \
    rust/scripts/vhdl_external_corpus_triage_gate.sh \
    rust/scripts/vhdl_stimuli_quality_gate.sh; do
    assert_tracked "$repo_file"
    assert_file_not_contains "$repo_file" "ebnf_to_json.pl"
  done

  assert_tracked "rust/scripts/ebnf_frontend_dual_run_diff_gate.sh"
  assert_tracked "rust/scripts/ebnf_frontend_readiness_gate.sh"
  assert_tracked "rust/scripts/ebnf_stimuli_quality_gate.sh"

  assert_file_contains \
    "rust/scripts/ebnf_frontend_dual_run_diff_gate.sh" \
    'perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$GRAMMARS_DIR/ebnf.ebnf" -o "$BOOTSTRAP_EBNF_JSON"'
  assert_file_contains \
    "rust/scripts/ebnf_frontend_dual_run_diff_gate.sh" \
    'if perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$perl_json"'

  assert_file_contains \
    "rust/scripts/ebnf_frontend_readiness_gate.sh" \
    'if [[ "$FRONTEND_IMPL" == "perl" ]]; then'
  assert_file_contains \
    "rust/scripts/ebnf_frontend_readiness_gate.sh" \
    'perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$json_out"'

  assert_file_contains \
    "rust/scripts/ebnf_stimuli_quality_gate.sh" \
    'if [[ "$FRONTEND_IMPL" == "perl" || "$require_ebnf_parseability" -eq 1 ]]; then'
  assert_file_contains \
    "rust/scripts/ebnf_stimuli_quality_gate.sh" \
    '"$EBNF_TO_JSON" --pretty --quiet "$EBNF_BOOTSTRAP_GRAMMAR" -o "$EBNF_BOOTSTRAP_JSON"'
}

audit_annotation_aggregate_contract_surface() {
  note "auditing aggregate annotation contract gate surface"

  assert_tracked "rust/scripts/annotation_robustness_gate.sh"
  assert_tracked "rust/scripts/annotation_stimuli_quality_gate.sh"

  assert_file_contains \
    "rust/Makefile" \
    'annotation_contract_gate - Enforce normative bootstrap annotation contracts + validator diagnostics'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_shared_contract_gate - Enforce shared bootstrap/generated annotation contracts'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_robustness_gate - Enforce advanced annotation suites + generated parseability/coverage checks'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_stimuli_quality_gate - Enforce strict closed-loop stimuli/coverage/gap checks (no-regression) for annotation grammars'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_runtime_contract_gate - Enforce semantic runtime/typed-AST contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_ast_roundtrip_gate - Enforce semantic AST round-trip contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_full_contract_gate - Enforce aggregate semantic contract gate (runtime + round-trip + regression)'
  assert_file_contains \
    "rust/Makefile" \
    'return_annotation_support_gate - Enforce aggregate 100% return-annotation support proof (audit + contract + stimuli)'
  assert_file_contains \
    "rust/Makefile" \
    'return_runtime_semantics_gate - Enforce typed return AST/runtime transform contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'return_ast_roundtrip_gate - Enforce canonical return AST round-trip contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'return_parity_gate - Enforce zero return mismatches on comparable (expectation-aligned) differential corpus'
  assert_file_contains \
    "rust/Makefile" \
    'return_full_contract_gate - Enforce aggregate return contract gate (runtime + round-trip + parity)'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib annotation_validator'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_builtin_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_builtin_contract'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_shared_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_full_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_full_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_robustness_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_stimuli_quality_gate'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib semantic_validator_'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_semantic_tree_to_ast_'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_usage_gate'

  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_runtime_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_ast_roundtrip_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_differential_regression_gate'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib unified_return_ast'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_return_tree_to_typed_ast_'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib return_validator'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib test_round_trip_runner'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_runtime_semantics_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_ast_roundtrip_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_parity_gate'

  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_shared_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_robustness_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_runtime_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_ast_roundtrip_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_full_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_runtime_semantics_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_ast_roundtrip_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_full_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_stimuli_quality_gate`'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_shared_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_usage_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_runtime_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_ast_roundtrip_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_full_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_robustness_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_stimuli_quality_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_annotation_support_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_runtime_semantics_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_parity_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_full_contract_gate` (local gate target)'

  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash annotation_contract_gate`'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash semantic_full_contract_gate`'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash return_annotation_support_gate`'

  assert_file_contains \
    "QUICKSTART_AI_ONBOARDING.md" \
    '`annotation_contract_gate`, `semantic_full_contract_gate`,'
  assert_file_contains \
    "QUICKSTART_AI_ONBOARDING.md" \
    '`return_annotation_support_gate`, and `annotation_stimuli_quality_gate`.'

  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '## Rust-To-Shell Contract Seams'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '- Aggregate annotation proof seam'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '- `annotation_contract_gate`'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '- `return_annotation_support_gate`'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'high-level entrypoints into aggregate annotation proof surfaces'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'operator-facing map of aggregate annotation / semantic / return local gates'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'annotation proof obligations and gate targets behind aggregate annotation claims'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is return/semantic annotation parsing or validation'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'And pick the nearest aggregate proof surface:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is proof plumbing, contract sidecars, or release-gate behavior'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'For annotation-specific proof plumbing, narrow quickly to:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'the nearest aggregate annotation proof surface:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'if the parser is `return_annotation` or `semantic_annotation`, usually also add:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-focused stimuli work, usually also add:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-proof changes, the practical aggregate readers are usually:'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'Symptom: Annotation-focused unit tests or leaf suites pass, but the repo-level annotation proof still feels wrong or incomplete'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'Annotation proof / closure problem'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'stopping at the leaf suite that passed instead of checking which aggregate proof claim the repo is actually making'
  assert_file_contains \
    "RUST_CODEBASE_ANALYSIS.md" \
    'patching the top-level proof claim before the nearest annotation seam is correct tends to hide whether the real drift is semantic behavior, closure evidence, or just the proof map'
}

audit_annotation_semantic_contract_surface() {
  note "auditing annotation semantic contract surface"

  assert_tracked "rust/scripts/sc01_contract_gate.sh"
  assert_tracked "rust/scripts/sc02_contract_gate.sh"
  assert_tracked "rust/scripts/sc03_contract_gate.sh"
  assert_tracked "rust/scripts/sc04_contract_gate.sh"
  assert_tracked "rust/scripts/sc05_contract_gate.sh"
  assert_tracked "rust/scripts/sc06_contract_gate.sh"
  assert_tracked "rust/scripts/sc07_contract_gate.sh"
  assert_tracked "rust/scripts/sc08_contract_gate.sh"
  assert_tracked "rust/scripts/sc09_contract_gate.sh"
  assert_tracked "rust/scripts/sc10_contract_gate.sh"
  assert_tracked "rust/scripts/sc11_contract_gate.sh"
  assert_tracked "rust/scripts/sc12_contract_gate.sh"
  assert_tracked "rust/scripts/sc13_contract_gate.sh"
  assert_tracked "rust/test_data/semantic_annotation/sc01_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc02_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc03_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc04_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc05_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc06_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc07_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc08_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc09_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc10_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc11_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc12_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc13_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sc01_contract_gate - Enforce SC-01 canonical-transform Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc02_contract_gate - Enforce SC-02 raw literal sample-hint Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc03_contract_gate - Enforce SC-03 directive routing + strict policy contract slices and differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc04_contract_gate - Enforce SC-04 token steering Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc05_contract_gate - Enforce SC-05 precedence/associativity Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc06_contract_gate - Enforce SC-06 branch weighting/selection Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc07_contract_gate - Enforce SC-07 recovery/sync Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc08_contract_gate - Enforce SC-08 value-domain Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc09_contract_gate - Enforce SC-09 relational-constraint Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc10_contract_gate - Enforce SC-10 coverage-target Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc11_contract_gate - Enforce SC-11 negative-case Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc12_contract_gate - Enforce SC-12 deterministic-partition Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc13_contract_gate - Enforce SC-13 profiles/runtime-scaffold Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc01_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc02_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc03_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc04_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc05_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc06_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc07_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc08_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc09_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc10_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc11_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc12_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc13_contract_gate'

  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc01_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc02_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc03_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc04_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc05_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc06_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc07_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc08_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc09_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc10_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc11_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc12_contract_gate`'
  assert_file_contains \
    "PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc13_contract_gate`'

  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc01_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc02_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc03_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc04_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc05_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc06_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc07_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc08_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc09_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc10_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc11_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc12_contract'
  assert_file_contains \
    "PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc13_contract'

  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    'semantic_annotation_sc01_contract'
  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    'target/sc01_contract_gate/work/sc01_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    '(.mismatched_cases == 0)'
  assert_file_not_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    'semantic_annotation_sc02_contract'
  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    'target/sc02_contract_gate/work/sc02_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    'semantic_annotation_sc03_contract'
  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    'target/sc03_contract_gate/work/sc03_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    'semantic_annotation_sc04_contract'
  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    'target/sc04_contract_gate/work/sc04_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    'semantic_annotation_sc05_contract'
  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    'target/sc05_contract_gate/work/sc05_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    'semantic_annotation_sc06_contract'
  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    'target/sc06_contract_gate/work/sc06_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    'semantic_annotation_sc07_contract'
  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    'target/sc07_contract_gate/work/sc07_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    'semantic_annotation_sc08_contract'
  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    'target/sc08_contract_gate/work/sc08_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    'semantic_annotation_sc09_contract'
  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    'target/sc09_contract_gate/work/sc09_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    'semantic_annotation_sc10_contract'
  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    'target/sc10_contract_gate/work/sc10_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    'semantic_annotation_sc11_contract'
  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    'target/sc11_contract_gate/work/sc11_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    'semantic_annotation_sc12_contract'
  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    'target/sc12_contract_gate/work/sc12_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    'semantic_annotation_sc13_contract'
  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    'target/sc13_contract_gate/work/sc13_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/test_data/semantic_annotation/sc01_contract.json" \
    '"generated_parser": "expected_fail"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc02_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc03_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc04_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc05_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc06_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc07_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc08_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc09_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc10_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc11_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc12_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc13_contract.json" \
    '"generated_parser": "pass"'
}

audit_sota_json_consumption_surface() {
  note "auditing aggregate SOTA summary.json consumption surface"

  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.roundtrip_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.roundtrip_contract_summary_json'

  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.proof_surfaces.family_contract_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.proof_surfaces.quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.vhdl.proof_surfaces.family_contract_summary_json'
}

audit_sota_nested_family_emission_surface() {
  note "auditing nested family SOTA summary.json emission surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'systemverilog: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'parser_aggregate_summary_json: maybe_path($sv_family_status_systemverilog_parser_aggregate_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'systemverilog: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'parser_aggregate_summary_json: maybe_path($sv_family_status_contract_systemverilog_parser_aggregate_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'semantic_scope_contract_summary_json: maybe_path($sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'regex: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'dual_run_summary_json: maybe_path($regex_family_dual_run_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'regex: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($regex_family_status_contract_regex_family_contract_summary_json)'

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'quality_parseability_report_json: maybe_path($vhdl_family_quality_parseability_report_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($vhdl_family_status_contract_vhdl_family_contract_summary_json)'
}

audit_combined_telemetry_nested_provenance_surface() {
  note "auditing combined telemetry nested provenance emission surface"

  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'state_dir: $sv_failure_context_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'failure_context_contract_summary_json: $sv_failure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'parser_aggregate_summary_json: $sv_family_status_systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'roundtrip_contract_summary_json: $sv_roundtrip_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'parser_aggregate_summary_json: $sv_family_status_contract_systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'semantic_scope_contract_summary_json: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'state_dir: $sv_roundtrip_contract_state_dir'

  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'dual_run_summary_json: $regex_family_dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'summary_json: $regex_family_status_contract_regex_family_contract_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'quality_parseability_report_json: $vhdl_family_quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'summary_json: $vhdl_family_status_contract_vhdl_family_contract_summary_json'
}

audit_sv_auxiliary_contract_surface() {
  note "auditing SV auxiliary-contract summary surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"
  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/sv_failure_context_contract_gate.sh"
  assert_tracked "rust/scripts/sv_roundtrip_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_state_dir: maybe_path($sv_failure_context_contract_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_summary_txt: maybe_path($sv_failure_context_contract_summary_txt)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_state_dir: maybe_path($sv_roundtrip_contract_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_summary_txt: maybe_path($sv_roundtrip_contract_summary_txt)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_failure_context_contract_summary_json: $sv_failure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_roundtrip_contract_summary_json: $sv_roundtrip_summary_json'

  assert_file_contains \
    "rust/scripts/sv_failure_context_contract_gate.sh" \
    'systemverilog_generation_counterexample_triage_json: $systemverilog_generation_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_failure_context_contract_gate.sh" \
    'systemverilog_preprocessor_counterexample_triage_json: $systemverilog_preprocessor_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_roundtrip_contract_gate.sh" \
    'systemverilog_roundtrip_initial_targets: $systemverilog_roundtrip_initial_targets'
  assert_file_contains \
    "rust/scripts/sv_roundtrip_contract_gate.sh" \
    'systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches: $systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches'
}

audit_family_layer_provenance_surface() {
  note "auditing family-layer provenance emission surface"

  assert_tracked "rust/scripts/sv_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/sv_parser_family_status_contract_gate.sh"
  assert_tracked "rust/scripts/regex_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/regex_parser_family_status_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_status_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'parser_aggregate_summary_json: $sv_parser_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'semantic_scope_contract_summary_json: $sv_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'aggregate_summary_json: $svpp_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'parser_aggregate_summary_json: $systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'semantic_scope_contract_summary_json: $systemverilog_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'aggregate_summary_json: $systemverilog_preprocessor_aggregate_summary_json'

  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'family_contract_state_dir: $regex_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'family_contract_summary_json: $regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'state_dir: $regex_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'summary_json: $regex_family_contract_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_summary_json: $vhdl_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'summary_json: $vhdl_family_contract_summary_json'
}

audit_family_summary_identity_surface() {
  note "auditing family-sidecar summary identity surface"

  for repo_file in \
    rust/scripts/sv_parser_aggregate_contract_gate.sh \
    rust/scripts/sv_preprocessor_aggregate_contract_gate.sh \
    rust/scripts/regex_parser_family_contract_gate.sh \
    rust/scripts/vhdl_parser_family_contract_gate.sh \
    rust/scripts/sv_parser_family_status_gate.sh \
    rust/scripts/regex_parser_family_status_gate.sh \
    rust/scripts/vhdl_parser_family_status_gate.sh \
    rust/scripts/sv_parser_family_status_contract_gate.sh \
    rust/scripts/regex_parser_family_status_contract_gate.sh \
    rust/scripts/vhdl_parser_family_status_contract_gate.sh; do
    assert_tracked "$repo_file"
    assert_file_contains "$repo_file" 'SUMMARY_JSON="$STATE_DIR/summary.json"'
    assert_file_contains "$repo_file" 'echo "summary_json: $SUMMARY_JSON"'
    assert_file_contains "$repo_file" '--arg summary_json "$SUMMARY_JSON"'
    assert_file_contains "$repo_file" 'state_dir: $state_dir'
    assert_file_contains "$repo_file" 'summary_txt: $summary_txt'
    assert_file_contains "$repo_file" 'summary_json: $summary_json'
  done
}

audit_family_contract_proof_surface() {
  note "auditing family-contract proof-surface emission surface"

  assert_tracked "rust/scripts/regex_parser_family_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'frontend_state_dir: $frontend_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'dual_run_summary_json: $dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_summary_csv: $stimuli_summary_csv'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'quality_parseability_report_json: $quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'strict_promotion_report_json: $strict_promotion_report_json'
}

audit_sv_aggregate_contract_proof_surface() {
  note "auditing SV aggregate-contract proof-surface emission surface"

  assert_tracked "rust/scripts/sv_parser_aggregate_contract_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'generation_report_json: $generation_report_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'shadow_report_json: $shadow_report_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'generation_counterexample_triage_json: $generation_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'replay_gap_target_triage_json: $replay_gap_target_triage_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'source_gap_json: $source_gap_json'

  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'quality_state_dir: $quality_state_dir'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'parseability_report_json: $parseability_report_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'counterexample_triage_json: $counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'gap_stage3_json: $gap_stage3_json'
}

audit_summary_json_emission_surface() {
  note "auditing top-level proof summary.json emission surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"
  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  for repo_file in \
    rust/scripts/sv_failure_context_contract_gate.sh \
    rust/scripts/sv_roundtrip_contract_gate.sh \
    rust/scripts/sota_exit_gate.sh \
    rust/scripts/sv_combined_telemetry_contract_gate.sh \
    rust/scripts/regex_combined_telemetry_contract_gate.sh \
    rust/scripts/vhdl_combined_telemetry_contract_gate.sh; do
    assert_file_contains "$repo_file" 'SUMMARY_JSON="$STATE_DIR/summary.json"'
    assert_file_contains "$repo_file" 'echo "generated_at_utc: $generated_at_utc"'
    assert_file_contains "$repo_file" 'echo "summary_json: $SUMMARY_JSON"'
    assert_file_contains "$repo_file" '--arg generated_at_utc "$generated_at_utc"'
    assert_file_contains "$repo_file" '--arg summary_json "$SUMMARY_JSON"'
    assert_file_contains "$repo_file" 'summary_json: $summary_json'
  done
}

assert_workflow_command() {
  local workflow_file="$1"
  local expected="$2"
  grep -F "$expected" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || \
    fail "workflow command drift detected in $workflow_file: expected '$expected'"
}

run_workflow() {
  local workflow_name="$1"
  local workflow_file="$2"
  local command_marker="$3"
  local command_line="$4"
  local log_file="$LOG_DIR/${workflow_name}.log"

  if ! is_selected "$workflow_name"; then
    note "skip $workflow_name (filtered)"
    return 0
  fi

  assert_workflow_command "$workflow_file" "$command_marker"
  note "run $workflow_name"
  if (
    cd "$EXPORT_DIR"
    export CARGO_NET_OFFLINE="$CARGO_OFFLINE_RAW"
    eval "$command_line"
  ) >"$log_file" 2>&1; then
    note "ok $workflow_name ($log_file)"
  else
    tail -n 120 "$log_file" >&2 || true
    fail "workflow local parity failed: $workflow_name (log: $log_file)"
  fi
}

main() {
  note "PGEN local CI workflow gate"
  note "run_dir: $RUN_DIR"
  note "filter: ${FILTER_RAW:-<all>}"
  note "cargo_offline: $CARGO_OFFLINE_RAW"

  require_tool git
  require_tool cargo
  require_tool make
  require_tool perl
  require_tool jq

  copy_tracked_worktree
  audit_static_include_paths
  audit_workflow_surface
  audit_ebnf_frontend_conversion_surface
  audit_annotation_aggregate_contract_surface
  audit_annotation_semantic_contract_surface
  audit_sota_json_consumption_surface
  audit_sota_nested_family_emission_surface
  audit_combined_telemetry_nested_provenance_surface
  audit_sv_auxiliary_contract_surface
  audit_family_layer_provenance_surface
  audit_family_summary_identity_surface
  audit_family_contract_proof_surface
  audit_sv_aggregate_contract_proof_surface
  audit_summary_json_emission_surface

  run_workflow \
    "annotation-contract-gate" \
    ".github/workflows/annotation-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash annotation_contract_gate" \
    "make -C rust SHELL=/bin/bash annotation_contract_gate"
  run_workflow \
    "annotation-nonbootstrap-e2e-gate" \
    ".github/workflows/annotation-nonbootstrap-e2e-gate.yml" \
    "make -C rust SHELL=/bin/bash annotation_nonbootstrap_e2e_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 make -C rust SHELL=/bin/bash annotation_nonbootstrap_e2e_gate"
  run_workflow \
    "branch-protection-contract-gate" \
    ".github/workflows/branch-protection-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash branch_protection_contract_gate" \
    "make -C rust SHELL=/bin/bash branch_protection_contract_gate"
  run_workflow \
    "differential-regression-gate" \
    ".github/workflows/differential-regression-gate.yml" \
    "make -C rust SHELL=/bin/bash differential_regression_gate" \
    "make -C rust SHELL=/bin/bash differential_regression_gate"
  run_workflow \
    "ebnf-frontend-dual-run-diff" \
    ".github/workflows/ebnf-frontend-dual-run-diff.yml" \
    "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff" \
    "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff"
  run_workflow \
    "fixed-point-gate" \
    ".github/workflows/fixed-point-gate.yml" \
    "make -C rust SHELL=/bin/bash fixed_point_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 make -C rust SHELL=/bin/bash fixed_point_gate"
  run_workflow \
    "performance-gate" \
    ".github/workflows/performance-gate.yml" \
    "make -C rust SHELL=/bin/bash performance_gate" \
    "make -C rust SHELL=/bin/bash performance_gate"
  run_workflow \
    "sota-exit-gate" \
    ".github/workflows/sota-exit-gate.yml" \
    "make -C rust SHELL=/bin/bash sota_exit_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 PGEN_SOTA_POLICY_FILE=$EXPORT_DIR/rust/config/sota_exit_policy.env PGEN_SOTA_RUN_EBNF_READINESS=1 PGEN_SOTA_REQUIRE_EBNF_STRICT=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=1 PGEN_SOTA_REQUIRE_EBNF_DUAL_RUN_STRICT=0 make -C rust SHELL=/bin/bash sota_exit_gate"

  note "all selected local workflow commands passed"
}

main "$@"
