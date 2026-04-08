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

audit_markdown_repo_relative_paths() {
  note "auditing markdown repo-path policy"
  if (cd "$ROOT_DIR" && rg -n --glob '*.md' '/Users/richarddje/Documents/github/pgen/' . >/dev/null 2>&1); then
    fail "absolute PGEN checkout path found in markdown docs; use relative repo paths"
  fi
}

audit_root_markdown_surface() {
  local -a expected_root_md=(
    "CHANGES.md"
    "COMMIT.md"
    "DEVELOPMENT_NOTES.md"
    "LIVE_ACHIEVEMENT_STATUS.md"
    "MEMORY.md"
    "PGEN_USER_GUIDE.md"
    "QUICKSTART_AI_ONBOARDING.md"
    "README.md"
    "SESSION_BOOTSTRAP.md"
  )
  local -a actual_root_md=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing root markdown allowlist"
  while IFS= read -r line; do
    actual_root_md+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\A[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_root_md[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_root_md[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected root markdown surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual root markdown surface:\n%s\n' "$actual_snapshot" >&2
    fail "root markdown allowlist drift detected; rehome stale root docs or update the tracked policy deliberately"
  fi
}

audit_top_level_docs_surface() {
  local -a expected_top_level_docs=(
    "docs/AST_GENERATOR_ARCHITECTURE.md"
    "docs/ast_transformation_pipeline.md"
    "docs/BOOTSTRAP_MODE_SPECIFICATION.md"
    "docs/EBNF_INCLUDE_SYSTEM.md"
    "docs/parser_architecture_evolution.md"
    "docs/RETURN_ANNOTATIONS_REFERENCE.md"
    "docs/TEST_INFRASTRUCTURE.md"
  )
  local -a actual_top_level_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing top-level docs allowlist"
  while IFS= read -r line; do
    actual_top_level_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_top_level_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_top_level_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected top-level docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual top-level docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "top-level docs allowlist drift detected; prune stale docs or update the tracked policy deliberately"
  fi
}

audit_contract_docs_surface() {
  local -a expected_contract_docs=(
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md"
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md"
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"
    "docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md"
  )
  local -a actual_contract_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing contract docs allowlist"
  while IFS= read -r line; do
    actual_contract_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/contracts\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_contract_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_contract_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected contract docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual contract docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "contract docs allowlist drift detected; rehome unexpected docs or update the tracked policy deliberately"
  fi
}

audit_reference_docs_surface() {
  local -a expected_reference_docs=(
    "docs/reference/PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md"
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md"
    "docs/reference/PGEN_RELEASE_POLICY.md"
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md"
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md"
    "docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md"
    "docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md"
    "docs/reference/RUST_CODEBASE_ANALYSIS.md"
    "docs/reference/STRESS_TEST_STANDARDIZATION.md"
    "docs/reference/SV_GRAMMAR_COVERAGE_MATRIX.md"
  )
  local -a actual_reference_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing reference docs allowlist"
  while IFS= read -r line; do
    actual_reference_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/reference\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_reference_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_reference_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected reference docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual reference docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "reference docs allowlist drift detected; rehome unexpected docs or update the tracked policy deliberately"
  fi
}

audit_active_docs_rehome_paths() {
  note "auditing active docs rehome paths"
  if (
    cd "$ROOT_DIR" &&
      rg -n \
        '(^|[^/])PGEN_SOTA_IMPLEMENTATION_ROADMAP\.md|(^|[^/])RUST_CODEBASE_ANALYSIS\.md|(^|[^/])PGEN_PARSER_INTEGRATION_CONTRACTS\.md|(^|[^/])PGEN_PARSER_ISSUE_REPORTING_PROTOCOL\.md|(^|[^/])PGEN_RELEASED_PARSER_BUG_LEDGER\.md|(^|[^/])PGEN_REGEX_PARSER_INTEGRATION_CONTRACT\.md|(^|[^/])PGEN_ANNOTATION_NORMATIVE_SPEC\.md|(^|[^/])PGEN_RELEASE_POLICY\.md|(^|[^/])PGEN_SEMANTIC_STEERING_CONTROL_MATRIX\.md|(^|[^/])PGEN_STIMULI_MODULE_NORMATIVE_SPEC\.md' \
        README.md \
        SESSION_BOOTSTRAP.md \
        QUICKSTART_AI_ONBOARDING.md \
        PGEN_USER_GUIDE.md \
        COMMIT.md \
        rust/docs/TECHNICAL_ARCHITECTURE.md \
        rust/docs/EMBEDDING_API_CONTRACT.md \
        rust/scripts/ci_workflow_local_gate.sh \
        docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md \
        docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md \
        docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md \
        docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md \
        docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md \
        docs/reference/PGEN_RELEASE_POLICY.md \
        docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md \
        docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md \
        docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md \
        docs/reference/RUST_CODEBASE_ANALYSIS.md \
        >/dev/null 2>&1
  ); then
    fail "active docs path drift detected; update live docs to use rehomed docs/contracts and docs/reference paths"
  fi
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
    .github/workflows/stimuli-cross-family-platform-gate.yml \
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
    .github/workflows/performance-gate.yml \
    .github/workflows/stimuli-cross-family-platform-gate.yml; do
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

audit_embedding_api_surface() {
  note "auditing public embedding API surface"

  assert_tracked "generated/regex.json"
  assert_tracked "generated/regex_parser.rs"
  assert_tracked "rust/src/embedding_api.rs"
  assert_tracked "rust/docs/EMBEDDING_API_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md"
  assert_tracked "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md"
  assert_tracked "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"
  assert_tracked "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "rust/scripts/regex_parser_integration_contract_gate.sh"
  assert_tracked "rust/scripts/regex_embedded_code_block_contract_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_parser_integration_contract_v1.json"
  assert_tracked "rust/test_data/grammar_quality/regex_embedded_code_block_contract_v0.json"

  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub const EMBEDDING_API_VERSION: &str = "1.2.0";'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_PARSER_INTEGRATION_CONTRACT_VERSION: &str = "1.1.8";'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_PARSER_RELEASE_VERSION: &str = "1.1.8";'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_AST_DUMP_SCHEMA_VERSION: u32 = 1;'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    '#[serde(rename = "regex")]'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    '#[serde(rename = "regex_default")]'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub struct ParseDiagnosticLocation {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub supports_regex_generated_backend: bool,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub stable_diagnostic_location_fields: Vec<String>,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub regex_parser_release_version: String,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub fn parse_regex_default(input: &str) -> GrammarParseOutcome {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub fn parse_regex_default_ast_dump('
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'GrammarFamily::Regex => parse_generated_regex(input),'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_generated_backend_enabled() -> bool {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_metadata_is_stable() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_failures_are_machine_localizable() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_accepts_declared_success_samples() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_classifies_whole_pattern_recursion_as_subroutine_call() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_classifies_numeric_backreferences() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_preserves_conditional_false_branch() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_binds_quantifier_to_final_literal_atom() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn parse_diagnostic_location_is_one_based_and_clamped_to_utf8_boundaries() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_rejects_declared_failure_samples() {'

  assert_file_contains \
    "rust/Makefile" \
    'regex_parser_integration_contract_gate - Validate regex parser integration contract (consumer-facing convenience API + diagnostics)'
  assert_file_contains \
    "rust/Makefile" \
    'regex_embedded_code_block_contract_gate - Validate regex embedded code-block structural contract over the checked-in synthetic corpus'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) regex_parser_integration_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_embedded_code_block_contract_gate.sh'
  assert_file_contains \
    "rust/scripts/regex_parser_integration_contract_gate.sh" \
    'cargo test --lib regex_parser_integration_contract_'
  assert_file_contains \
    "rust/scripts/regex_parser_integration_contract_gate.sh" \
    'cargo test --features generated_parsers --lib regex_parser_integration_contract_'
  assert_file_contains \
    "rust/scripts/regex_embedded_code_block_contract_gate.sh" \
    'regex_embedded_code_block_contract_v0.json'
  assert_file_contains \
    "rust/scripts/regex_embedded_code_block_contract_gate.sh" \
    '--parse "$expected_parser_type" "$case_input_file" --profile "$expected_profile"'

  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    'Provide a stable, versioned surface for external projects embedding PGEN annotation parsing and selected grammar parsing'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`parse_regex_default(...)`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`parse_regex_default_ast_dump(...)`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`make -C rust regex_parser_integration_contract_gate`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`GrammarFamily`: `systemverilog | vhdl | regex`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`GrammarProfile`: `sv_2017 | sv_2023 | vhdl_1076_2019 | regex_default`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`ParseDiagnostic`: stable `code` + human-readable `message` + optional `location`.'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`regex_parser_release_version`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`regex_generated_backend_required_feature`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`embedding_api_gate` now covers the public regex parser/profile surface too.'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `regex`: `regex_default`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `parse_regex_default(...)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `parse_regex_default_ast_dump(...)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'the public embedding API now exposes regex through `regex_default`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_parser_integration_contract_gate'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '### Regex Parser Flavor'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`parseability_attempts_total=1554`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?<name>[a-z]+)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`[[:^alnum:]]+`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?<A>foo)-\\k{A}`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`a{,4}`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?{lua: return x + 1})`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?{javascript:return x + 1;})`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'plain `(?{...})` is preserved as opaque generic payload'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'language tags `lua`, `js`, `javascript`, and `rhai` are preserved as opaque source-body payloads'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_embedded_code_block_contract_gate'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'this is not a host-language regex literal parser for wrapper forms such as `/pattern/flags`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- whole-pattern recursion via `(?R)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- explicit false branches are preserved separately, so `(?(1)a|b)` transports `a` and `b` as distinct yes/no branches'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '  - `(?R)` now appears as `subroutine_call` / `subroutine_target`, not `inline_modifiers`'

  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'Every current and future parser family that PGEN publishes for downstream consumption must have a tracked integration-contract document.'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'its family document must publish:'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '- `Contract Identity`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '| `regex` | `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Downstream-ready regex contract for RGX and other regex consumers; current published release `1.1.8`. |'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'Accepted reports should then be logged in:'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'If one or more downstream consumer repos also track the same issue locally'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'The parser family/profile is the primary tracking axis for released-parser support.'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'for regex, copy `parser_embedding_api_contract().regex_parser_release_version`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'for regex, copy `parser_embedding_api_contract().regex_integration_contract_version`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'Every downstream bug report against a released parser family must receive a stable report ID.'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    '`Downstream Tracking Refs`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    '`Reported Against Parser Release`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'The primary index for this ledger is `Parser Family/Profile`.'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'the parser release version containing the fix'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '- Contract version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '- Parser release version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_parser_integration_contract_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Issue Reporting Quick Path'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'PGEN_TRACE_VERBOSITY=debug'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Regex AST-dump schema version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'stable optional machine-localizable location object'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Published Regex Flavor Summary'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '## Release 1.1.8 Highlights'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'raw regex bodies, not host-language delimiter wrappers'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'braced named backreferences such as `\k{name}`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'subroutine-reference forms such as `\g{1}` and `\g<1>`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'whole-pattern recursion `(?R)` now classifies as `subroutine_call` / `subroutine_target`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'numeric backreferences such as `\1` now classify as `backreference` instead of generic `escape`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'negated POSIX classes such as `[[:^alnum:]]`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'explicit conditional false branches such as `(?(1)a|b)` now preserve separate `yes_branch` and `no_branch` spans'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'plain `(?{...})` is preserved as opaque generic payload'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '`lua`, `js`, `javascript`, and `rhai` payloads are preserved as opaque source-body payloads'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_embedded_code_block_contract_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'
  assert_file_not_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'generated/regex.json'
  assert_file_not_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'grammars/regex.ebnf'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'local git-tracked records in PGEN plus zero-or-more downstream consumer repos are sufficient'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'
  assert_file_contains \
    "COMMIT.md" \
    '`docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md` and `docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md`'
  assert_file_contains \
    "COMMIT.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'

  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'internal parser-registry or probe availability automatically means the same family already has a public embedding contract'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'It now also has a public embedding seam in `embedding_api.rs`, but that public surface should not be mistaken for complete parser-family closure by itself'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'dedicated downstream integration contract doc plus a regex-specific host contract gate'
}

audit_rtl_frontend_generated_contract_surface() {
  note "auditing rtl_frontend generated contract surface"

  assert_tracked ".github/workflows/rtl-frontend-generated-contract-gate.yml"
  assert_tracked "generated/rtl_frontend.json"
  assert_tracked "generated/rtl_frontend_parser.rs"
  assert_tracked "grammars/rtl_frontend.ebnf"
  assert_tracked "rust/src/bin/rtl_frontend_generated_contract_probe.rs"
  assert_tracked "rust/src/parser_registry.rs"
  assert_tracked "rust/scripts/rtl_frontend_generated_contract_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json"

  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'rtl_frontend_generated_parity_contract_v0.json'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'parse_sample("rtl_frontend", &sample.sample)'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'parse_sample_ast_json("rtl_frontend", &sample.sample)'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'required_rule_names'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'forbidden_rule_names'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'expected_rule_texts'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'fn rtl_frontend_generated_contract_metadata_is_stable() {'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'fn rtl_frontend_generated_contract_samples_hold() {'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"required_rule_names"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"forbidden_rule_names"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"expected_rule_texts"'
  assert_file_contains \
    "rust/scripts/rtl_frontend_generated_contract_gate.sh" \
    'cargo run --features generated_parsers --bin rtl_frontend_generated_contract_probe'
  assert_file_contains \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    'make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate'
  assert_file_contains \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    'path: rust/target/rtl_frontend_generated_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    'rtl_frontend_generated_contract_gate - Validate curated generated rtl_frontend parseability/AST contract samples'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/rtl_frontend_generated_contract_gate.sh'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate`'
}

audit_stimuli_cross_family_platform_surface() {
  note "auditing cross-family stimuli platform surface"

  assert_tracked ".github/workflows/stimuli-cross-family-platform-gate.yml"
  assert_tracked "rust/scripts/stimuli_cross_family_platform_gate.sh"
  assert_tracked "rust/scripts/ebnf_stimuli_quality_gate.sh"
  assert_tracked "rust/scripts/vhdl_stimuli_quality_gate.sh"
  assert_tracked "rust/scripts/sv_stimuli_quality_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_family_stimuli_contract.json"
  assert_tracked "rust/test_data/grammar_quality/vhdl_stimuli_cross_family_platform_contract_v0.json"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json"

  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_CONTRACT="${REGEX_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_COUNT="${REGEX_COUNT}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${REGEX_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_QUALITY_CONTRACT="${VHDL_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_CARGO_TARGET_DIR=target'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${VHDL_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_CONTRACT="${SV_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_LRM_PROFILES="${SV_LRM_PROFILES}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'CROSS_FAMILY_CARGO_BUILD_JOBS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_CARGO_BUILD_JOBS:-1}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${SV_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    'make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate'
  assert_file_contains \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    'path: rust/target/stimuli_cross_family_platform_gate'
  assert_file_contains \
    "rust/Makefile" \
    'stimuli_cross_family_platform_gate - Validate bounded shared stimuli-platform replay across regex, VHDL, and SystemVerilog'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/stimuli_cross_family_platform_gate.sh'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate`'
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
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_shared_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_robustness_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_runtime_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_ast_roundtrip_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_full_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_runtime_semantics_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_ast_roundtrip_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_full_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
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
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '## Rust-To-Shell Contract Seams'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- Aggregate annotation proof seam'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- `annotation_contract_gate`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- `return_annotation_support_gate`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'high-level entrypoints into aggregate annotation proof surfaces'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'operator-facing map of aggregate annotation / semantic / return local gates'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'annotation proof obligations and gate targets behind aggregate annotation claims'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'If an annotation leaf suite or one SC gate passes, the repo-level annotation proof claim is done.'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Aggregate annotation proof composition'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Operator-facing annotation gate map'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Aggregate annotation proof contract'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is return/semantic annotation parsing or validation'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'And pick the nearest aggregate proof surface:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is proof plumbing, contract sidecars, or release-gate behavior'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'For annotation-specific proof plumbing, narrow quickly to:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'the nearest aggregate annotation proof surface:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'if the parser is `return_annotation` or `semantic_annotation`, usually also add:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-focused stimuli work, usually also add:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-proof changes, the practical aggregate readers are usually:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Symptom: Annotation-focused unit tests or leaf suites pass, but the repo-level annotation proof still feels wrong or incomplete'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Annotation proof / closure problem'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'stopping at the leaf suite that passed instead of checking which aggregate proof claim the repo is actually making'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
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
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc01_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc02_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc03_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc04_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc05_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc06_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc07_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc08_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc09_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc10_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc11_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc12_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc13_contract_gate`'

  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc01_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc02_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc03_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc04_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc05_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc06_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc07_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc08_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc09_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc10_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc11_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc12_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
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
    '.family_status.regex.primary_unmet_closure_criterion // "<none>"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.stimuli_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.proof_surfaces.family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.primary_unmet_detail_criterion // "<none>"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.proof_surfaces.formal_exhaustive_closure_summary_json'

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
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.proof_surfaces.formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.vhdl.proof_surfaces.formal_exhaustive_closure_summary_json'
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
    'formal_exhaustive_closure_summary_json: maybe_path($sv_family_status_systemverilog_formal_exhaustive_closure_summary_json)'
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
    'formal_exhaustive_closure_summary_json: maybe_path($sv_family_status_contract_systemverilog_formal_exhaustive_closure_summary_json)'
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
    'stimuli_parseability_report_json: maybe_path($regex_family_stimuli_parseability_report_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: maybe_path($regex_family_stimuli_parseability_counterexample_triage_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'dual_run_summary_json: maybe_path($regex_family_dual_run_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_state_dir: maybe_path($regex_family_status_regex_formal_exhaustive_closure_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($regex_family_status_regex_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'regex: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($regex_family_status_contract_regex_family_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_state_dir: maybe_path($regex_family_status_contract_regex_formal_exhaustive_closure_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($regex_family_status_contract_regex_formal_exhaustive_closure_summary_json)'

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'quality_parseability_report_json: maybe_path($vhdl_family_quality_parseability_report_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($vhdl_family_status_vhdl_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($vhdl_family_status_contract_vhdl_family_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($vhdl_family_status_contract_vhdl_formal_exhaustive_closure_summary_json)'
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
    'formal_exhaustive_closure_summary_json: $sv_family_status_systemverilog_formal_exhaustive_closure_summary_json'
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
    'formal_exhaustive_closure_summary_json: $sv_family_status_contract_systemverilog_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'state_dir: $sv_roundtrip_contract_state_dir'

  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'dual_run_summary_json: $regex_family_dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_report_json: $regex_family_stimuli_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: $regex_family_stimuli_parseability_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_parser_rejections_zero: $regex_family_status_regex_stimuli_parseability_parser_rejections_zero'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_counterexample_primary_parser_error: $regex_family_stimuli_parseability_counterexample_primary_parser_error'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'summary_json: $regex_family_status_contract_regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'state_dir: $regex_family_status_regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'formal_exhaustive_closure_summary_json: $regex_family_status_regex_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'state_dir: $regex_family_status_contract_regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'summary_json: $regex_family_status_contract_regex_formal_exhaustive_closure_summary_json'

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
    'PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_SKIP_FAMILY_STATUS=1'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $sv_formal_exhaustive_closure_summary_json'
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
    'formal_exhaustive_closure_summary_json: $systemverilog_formal_exhaustive_closure_summary_json'
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
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_state_dir: $regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $regex_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'state_dir: $regex_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'summary_json: $regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'state_dir: $regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'summary_json: $regex_formal_exhaustive_closure_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_summary_json: $vhdl_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_state_dir: $vhdl_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $vhdl_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'summary_json: $vhdl_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'state_dir: $vhdl_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'summary_json: $vhdl_formal_exhaustive_closure_summary_json'
}

audit_family_summary_identity_surface() {
  note "auditing family-sidecar summary identity surface"

  for repo_file in \
    rust/scripts/sv_parser_aggregate_contract_gate.sh \
    rust/scripts/sv_preprocessor_aggregate_contract_gate.sh \
    rust/scripts/sv_formal_exhaustive_closure_gate.sh \
    rust/scripts/regex_parser_family_contract_gate.sh \
    rust/scripts/regex_broader_corpus_proof_gate.sh \
    rust/scripts/regex_formal_exhaustive_closure_gate.sh \
    rust/scripts/vhdl_parser_family_contract_gate.sh \
    rust/scripts/vhdl_formal_exhaustive_closure_gate.sh \
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
  assert_tracked "rust/test_data/grammar_quality/regex_family_stimuli_contract.json"

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_family_stimuli_contract.json" \
    '"grammar_name": "regex"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_family_stimuli_contract.json" \
    '"require_parseability": true'

  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'STIMULI_CONTRACT_FILE="${PGEN_REGEX_FAMILY_CONTRACT_STIMULI_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/regex_family_stimuli_contract.json}"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'require_file "$STIMULI_CONTRACT_FILE"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_CONTRACT="$STIMULI_CONTRACT_FILE"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'frontend_state_dir: $frontend_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_contract_file: $stimuli_contract_file'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'dual_run_summary_json: $dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_summary_csv: $stimuli_summary_csv'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_parseability_report_json: $stimuli_regex_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: $stimuli_regex_parseability_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_regex_parseability_parser_rejections_total: $stimuli_regex_parseability_parser_rejections_total'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_regex_parseability_counterexample_primary_parser_error: $stimuli_regex_parseability_counterexample_primary_parser_error'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'quality_parseability_report_json: $quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'strict_promotion_report_json: $strict_promotion_report_json'
}

audit_regex_corpus_bundle_surface() {
  note "auditing regex corpus bundle surface"

  assert_tracked "regex_corpus_bundle/README.md"
  assert_tracked "regex_corpus_bundle/docs/regex_corpus_plan.md"
  assert_tracked "regex_corpus_bundle/manifests/upstreams.lock.json"
  assert_tracked "regex_corpus_bundle/manifests/licenses.json"
  assert_tracked "regex_corpus_bundle/schemas/regex_case.schema.json"
  assert_tracked "regex_corpus_bundle/scripts/fetch_regex_corpora.py"
  assert_tracked "regex_corpus_bundle/corpus/pcre2/invalid/README.md"
  assert_tracked "regex_corpus_bundle/corpus/pcre2/quarantine/README.md"
  assert_tracked "regex_corpus_bundle/oracle/pcre2/README.md"
  assert_tracked "rust/scripts/regex_corpus_bundle_contract_gate.sh"

  assert_file_contains \
    "rust/Makefile" \
    'regex_corpus_bundle_contract_gate - Validate the tracked PCRE2-first regex corpus acquisition bundle contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_corpus_bundle_contract_gate.sh'

  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'BUNDLE_DIR="$ROOT_DIR/regex_corpus_bundle"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'LOCKFILE="$BUNDLE_DIR/manifests/upstreams.lock.json"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'SCHEMA_FILE="$BUNDLE_DIR/schemas/regex_case.schema.json"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'run_logged "fetch_regex_corpora_help" python3 "$FETCH_SCRIPT" --help'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'status_effect: does_not_reopen_closed_regex_family_row'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'maintained PCRE2-first acquisition/inventory starter for future regex hardening'

  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'Repo-ready starter bundle for a **PCRE2-first** regex corpus pipeline.'
  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'make -C rust regex_corpus_bundle_contract_gate'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '1. **Canonical syntax and behavior source:** PCRE2 upstream `testdata/testinput*` and related files.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '2. **Secondary PCRE2-relevant source:** PHP `ext/pcre/tests`, because PHP uses PCRE2 but wraps patterns in PHP-specific delimiters and modifiers.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '3. **Non-goal for phase 1:** treating non-PCRE2 engines as syntax ground truth.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    'make -C rust regex_corpus_bundle_contract_gate'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '### Regex External Corpus Hardening'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`regex_corpus_bundle/`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_corpus_bundle_contract_gate'
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`regex_corpus_bundle/` is the canonical PCRE2-first starter for widening regex evidence'
  assert_file_contains \
    "LIVE_ACHIEVEMENT_STATUS.md" \
    'future regex hardening now also has a maintained external-corpus acquisition lane under `regex_corpus_bundle/`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`regex_corpus_bundle/`'
  assert_file_contains \
    "README.md" \
    '`regex_corpus_bundle/`: PCRE2-first regex corpus acquisition/inventory starter for future regex hardening'
}

audit_regex_pcre2_compile_oracle_surface() {
  note "auditing regex PCRE2 compile-oracle surface"

  assert_tracked "regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py"
  assert_tracked "rust/scripts/regex_pcre2_compile_oracle_gate.sh"
  assert_tracked "rust/src/bin/regex_corpus_probe.rs"
  assert_tracked "rust/src/regex_compile_validation.rs"
  assert_tracked "rust/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env"

  assert_file_contains \
    "rust/Makefile" \
    'regex_pcre2_compile_oracle_gate - Normalize canonical PCRE2 compile-oracle cases from testinput2/testoutput2 and enforce tracked mismatch ceilings'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_pcre2_compile_oracle_gate.sh'

  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'scripts/normalize_pcre2_compile_oracle.py'
  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '`normalize_pcre2_compile_oracle.py`'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'

  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'NORMALIZER="$BUNDLE_DIR/scripts/normalize_pcre2_compile_oracle.py"'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'BASELINE_ENV="${PGEN_REGEX_PCRE2_COMPILE_ORACLE_BASELINE_ENV:-$RUST_DIR/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env}"'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'expected_parse_ok_total: $expected_parse_ok_total'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'false_accept_total: $false_accept_total'
  assert_file_contains \
    "rust/src/regex_compile_validation.rs" \
    'pub fn validate_regex_compile_contract(input: &str) -> Result<(), RegexCompileValidationError> {'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'validate_regex_compile_contract(&owned_input)'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'the compile-oracle gate is the first external-corpus lane that actually measures expected compile outcomes against PCRE2 source truth'
  assert_file_contains \
    "LIVE_ACHIEVEMENT_STATUS.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate` pairs PCRE2 `testinput2` with `testoutput2`'
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate` consumes the new normalizer `regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`regex_pcre2_compile_oracle_gate` for compile-truth comparison against pinned PCRE2 source truth'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`rust/src/regex_compile_validation.rs`'
  assert_file_contains \
    "README.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate`'
}

audit_regex_formal_exhaustive_closure_surface() {
  note "auditing regex formal exhaustive-closure surface"

  assert_tracked "rust/scripts/regex_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/regex_broader_corpus_proof_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/regex_broader_corpus_v0.json"

  assert_file_contains \
    "rust/Makefile" \
    'regex_broader_corpus_proof_gate - Run deterministic broader regex corpus proof over the checked-in regex stress corpus'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_broader_corpus_proof_gate.sh'
  assert_file_contains \
    "rust/Makefile" \
    'regex_formal_exhaustive_closure_gate - Compute the explicit regex exhaustive-closure proof surface status from the family sidecar'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"source_file": "rust/test_data/regex/stress_tests.json"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"expected_case_count": 44'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"expected_parser_type": "regex"'

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "broader_corpus_backed_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "Regex still lacks a checked-in broader corpus-backed proof surface'

  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'MANIFEST_FILE="${PGEN_REGEX_BROADER_CORPUS_PROOF_MANIFEST:-$RUST_DIR/test_data/grammar_quality/regex_broader_corpus_v0.json}"'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'cargo build --features generated_parsers --bin parseability_probe'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    '"$PARSE_PROBE_BIN" --supports regex'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'parse_fail_total'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'primary_parse_failure_case'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'REGEX_BROADER_CORPUS_PROOF_GATE="$RUST_DIR/scripts/regex_broader_corpus_proof_gate.sh"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'run_logged "regex_broader_corpus_proof_gate"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_broader_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'REGEX_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/regex_parser_family_contract_gate.sh"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'EXISTING_BROADER_CORPUS_PROOF_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_BROADER_CORPUS_PROOF_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'broader_corpus_backed_proof_surface_present'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_unmet+=("${required_surface_key}=missing")'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'broader_corpus_backed_proof_summary_json'
}

audit_vhdl_formal_exhaustive_closure_surface() {
  note "auditing VHDL formal exhaustive-closure surface"

  assert_tracked "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/vhdl_external_corpus_triage_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/vhdl_external_corpus_triage_v0.json"

  assert_file_contains \
    "rust/Makefile" \
    'vhdl_formal_exhaustive_closure_gate - Compute the explicit VHDL exhaustive-closure proof surface status from family and external-corpus sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/vhdl_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "external_corpus_backed_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "VHDL still lacks a checked-in external corpus-backed proof surface'

  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'VHDL_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/vhdl_external_corpus_triage_gate.sh"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'run_logged "vhdl_external_corpus_triage_gate"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'vhdl_external_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'vhdl_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'external_corpus_backed_proof_summary_json'
}

audit_sv_formal_exhaustive_closure_surface() {
  note "auditing SV formal exhaustive-closure surface"

  assert_tracked "rust/scripts/sv_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/sv_external_corpus_triage_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sv_formal_exhaustive_closure_gate - Compute the explicit SV exhaustive-closure proof surface status from family-status and external-corpus sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "external_corpus_backed_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "SystemVerilog still lacks an explicit checked-in external corpus-backed proof surface sidecar'

  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'SV_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/sv_external_corpus_triage_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'SV_FAMILY_STATUS_GATE="$RUST_DIR/scripts/sv_parser_family_status_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'run_logged "sv_external_corpus_triage_gate"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_external_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_external_corpus_backed_proof_summary_json'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust SHELL=/bin/bash sv_formal_exhaustive_closure_gate'
  assert_file_contains \
    "LIVE_ACHIEVEMENT_STATUS.md" \
    '`make -C rust SHELL=/opt/homebrew/bin/bash sv_formal_exhaustive_closure_gate` now computes an explicit external-corpus-backed formal-closure sidecar'
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`sv_formal_exhaustive_closure_gate` now makes the missing-vs-present SystemVerilog external-corpus proof surface explicit'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`sv_formal_exhaustive_closure_gate` when the task is SystemVerilog external-corpus proof normalization'
}

audit_sv_preprocessor_formal_exhaustive_closure_surface() {
  note "auditing SV preprocessor formal exhaustive-closure surface"

  assert_tracked "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_syntax_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_reachability_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sv_preprocessor_zero_plausible_gap_proof_gate - Prove the SV-preprocessor helper-only syntax-unreachable whitelist over the retained aggregate/reachability sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'sv_preprocessor_formal_exhaustive_closure_gate - Compute the explicit SV-preprocessor exhaustive-closure proof surface status from syntax, aggregate, and reachability sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "zero_plausible_grammar_level_gap_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "SystemVerilog preprocessor still lacks an explicit grammar-level exhaustive proof surface'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json" \
    '"allowed_unreachable_rules":'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json" \
    '"helper_only_whitelist_detail": "SystemVerilog preprocessor zero-plausible-gap proof requires the syntax-unreachable surface'

  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'AGGREGATE_CONTRACT_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'REACHABILITY_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'ZERO_GAP_PROOF_GATE="$RUST_DIR/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_SYNTAX_CLOSURE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_AGGREGATE_CONTRACT_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_REACHABILITY_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_ZERO_GAP_PROOF_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_ZERO_PLAUSIBLE_GAP_PROOF_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'aggregate_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'reachability_closure_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'zero_gap_proof_summary_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'zero_plausible_grammar_level_gap_proof_surface'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh" \
    'helper_only_unreachable_surface_green'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh" \
    'zero_plausible_grammar_level_gap_proof_surface'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust SHELL=/bin/bash sv_preprocessor_formal_exhaustive_closure_gate'
  assert_file_contains \
    "LIVE_ACHIEVEMENT_STATUS.md" \
    '`make -C rust SHELL=/opt/homebrew/bin/bash sv_preprocessor_formal_exhaustive_closure_gate`'
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`sv_preprocessor_formal_exhaustive_closure_gate` now makes the missing-vs-present SystemVerilog-preprocessor grammar-level proof surface explicit'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`sv_preprocessor_formal_exhaustive_closure_gate` when the task is SystemVerilog-preprocessor formal-closure proof normalization'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_gate'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_family_status_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_family_status_contract_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_family_status_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_family_status_contract_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
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
  audit_markdown_repo_relative_paths
  audit_root_markdown_surface
  audit_top_level_docs_surface
  audit_contract_docs_surface
  audit_reference_docs_surface
  audit_active_docs_rehome_paths
  audit_workflow_surface
  audit_ebnf_frontend_conversion_surface
  audit_embedding_api_surface
  audit_rtl_frontend_generated_contract_surface
  audit_stimuli_cross_family_platform_surface
  audit_annotation_aggregate_contract_surface
  audit_annotation_semantic_contract_surface
  audit_sota_json_consumption_surface
  audit_sota_nested_family_emission_surface
  audit_combined_telemetry_nested_provenance_surface
  audit_sv_auxiliary_contract_surface
  audit_family_layer_provenance_surface
  audit_family_summary_identity_surface
  audit_family_contract_proof_surface
  audit_regex_corpus_bundle_surface
  audit_regex_pcre2_compile_oracle_surface
  audit_regex_formal_exhaustive_closure_surface
  audit_sv_formal_exhaustive_closure_surface
  audit_sv_preprocessor_formal_exhaustive_closure_surface
  audit_vhdl_formal_exhaustive_closure_surface
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
    "rtl-frontend-generated-contract-gate" \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate" \
    "make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate"
  run_workflow \
    "stimuli-cross-family-platform-gate" \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    "make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate" \
    "make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate"
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
