#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC02_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc02_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc02_semantic_differential_report.json"

mkdir -p "$LOG_DIR" "$WORK_DIR"

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 40 "$log_file" >&2 || true
        exit 1
    fi
}

echo "==> SC-02 contract gate"
echo "state_dir: $STATE_DIR"

run_logged_rust "semantic_usage_sc02_raw_literal_hint_contract" \
    cargo test --lib semantic_usage_stimuli_raw_quoted_content_returns_literal_hint
run_logged_rust "semantic_usage_sc02_literalish_directives_contract" \
    cargo test --lib semantic_usage_stimuli_literalish_directives_accept_structured_and_legacy_alias_hints
run_logged_rust "semantic_usage_sc02_named_guard_contract" \
    cargo test --lib semantic_usage_stimuli_raw_hint_requires_literalish_directive_when_named

run_logged_rust "bootstrap_sc02_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc02_contract
run_logged_rust "generated_sc02_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc02_contract

run_logged_rust "differential_sc02_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc02_contract --differential-comparable-only --differential-report-json "target/sc02_contract_gate/work/sc02_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-02 differential report at '$REPORT_JSON'" >&2
    exit 1
fi

jq -e '
  (.total_cases > 0) and
  (.mismatched_cases == 0) and
  (((.mismatch_category_counts // {}) | to_entries | map(.value) | add) // 0 == (.mismatched_cases // 0)) and
  (
    (((.mismatch_category_counts // {}) | keys) -
      ["baseline_success_candidate_failure", "baseline_failure_candidate_success", "normalized_output_mismatch"]
    ) | length
  ) == 0
' "$REPORT_JSON" >/dev/null

cat <<EOF
✅ SC-02 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
