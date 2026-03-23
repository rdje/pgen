#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_FAMILY_CONTRACT_STATE_DIR:-$RUST_DIR/target/vhdl_parser_family_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

VHDL_QUALITY_GATE="$RUST_DIR/scripts/vhdl_stimuli_quality_gate.sh"
VHDL_STRICT_PROMOTION_GATE="$RUST_DIR/scripts/vhdl_strict_promotion_gate.sh"

EXISTING_QUALITY_STATE_DIR="${PGEN_VHDL_FAMILY_CONTRACT_EXISTING_QUALITY_STATE_DIR:-}"
EXISTING_STRICT_PROMOTION_STATE_DIR="${PGEN_VHDL_FAMILY_CONTRACT_EXISTING_STRICT_PROMOTION_STATE_DIR:-}"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
        exit 1
    fi
}

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "error: missing required file '$path'" >&2
        exit 1
    fi
}

require_nonempty_file() {
    local path="$1"
    if [[ ! -s "$path" ]]; then
        echo "error: expected non-empty artifact '$path'" >&2
        exit 1
    fi
}

extract_summary_value() {
    local path="$1"
    local key="$2"
    local line
    line="$(grep -F "${key}: " "$path" | tail -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: missing key '${key}' in summary '$path'" >&2
        exit 1
    fi
    printf '%s\n' "${line#${key}: }"
}

assert_equal() {
    local label="$1"
    local expected="$2"
    local actual="$3"
    if [[ "$expected" != "$actual" ]]; then
        echo "error: ${label} mismatch: expected '${expected}' but found '${actual}'" >&2
        exit 1
    fi
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

require_tool jq
require_file "$VHDL_QUALITY_GATE"
require_file "$VHDL_STRICT_PROMOTION_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

quality_state_dir="${EXISTING_QUALITY_STATE_DIR:-$WORK_DIR/vhdl_stimuli_quality_gate}"
strict_promotion_state_dir="${EXISTING_STRICT_PROMOTION_STATE_DIR:-$WORK_DIR/vhdl_strict_promotion_gate}"

if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    run_logged "vhdl_stimuli_quality_gate" env \
        PGEN_VHDL_STIMULI_QUALITY_STATE_DIR="$quality_state_dir" \
        "$VHDL_QUALITY_GATE"
fi

if [[ -z "$EXISTING_STRICT_PROMOTION_STATE_DIR" ]]; then
    run_logged "vhdl_strict_promotion_gate" env \
        PGEN_VHDL_STRICT_PROMOTION_STATE_DIR="$strict_promotion_state_dir" \
        "$VHDL_STRICT_PROMOTION_GATE"
fi

quality_summary_txt="$quality_state_dir/summary.txt"
quality_realistic_report_json="$quality_state_dir/work/vhdl_realistic_corpus_report.json"
quality_parseability_report_json="$quality_state_dir/work/vhdl_parseability_generation_report.json"
strict_promotion_summary_txt="$strict_promotion_state_dir/summary.txt"
strict_promotion_report_json="$strict_promotion_state_dir/work/vhdl_strict_promotion_report.json"

require_nonempty_file "$quality_summary_txt"
require_nonempty_file "$quality_realistic_report_json"
require_nonempty_file "$quality_parseability_report_json"
require_nonempty_file "$strict_promotion_summary_txt"
require_nonempty_file "$strict_promotion_report_json"

quality_contract_file="$(extract_summary_value "$quality_summary_txt" "contract_file")"
quality_closed_loop_initial_status="$(extract_summary_value "$quality_summary_txt" "closed_loop_initial_status")"
quality_closed_loop_replay_status="$(extract_summary_value "$quality_summary_txt" "closed_loop_replay_status")"
quality_closed_loop_initial_targets="$(extract_summary_value "$quality_summary_txt" "closed_loop_initial_targets")"
quality_closed_loop_replay_targets="$(extract_summary_value "$quality_summary_txt" "closed_loop_replay_targets")"
quality_closed_loop_parseability_shadow_requested_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_requested_total")"
quality_closed_loop_parseability_shadow_attempts_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_attempts_total")"
quality_closed_loop_parseability_shadow_accepted_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_accepted_total")"
quality_closed_loop_parseability_shadow_rejected_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_rejected_total")"
quality_closed_loop_parseability_shadow_parser_rejections_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_parser_rejections_total")"
quality_closed_loop_parseability_shadow_generation_errors_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_generation_errors_total")"
quality_closed_loop_parseability_shadow_empty_generations_total="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_empty_generations_total")"
quality_closed_loop_parseability_shadow_acceptance_rate_percent="$(extract_summary_value "$quality_summary_txt" "closed_loop_parseability_shadow_acceptance_rate_percent")"
quality_parseability_generation_requested_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_requested_total")"
quality_parseability_generation_attempts_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_attempts_total")"
quality_parseability_generation_accepted_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_accepted_total")"
quality_parseability_generation_rejected_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_rejected_total")"
quality_parseability_generation_parser_rejections_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_parser_rejections_total")"
quality_parseability_generation_errors_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_errors_total")"
quality_parseability_generation_empty_generations_total="$(extract_summary_value "$quality_summary_txt" "parseability_generation_empty_generations_total")"
quality_parseability_generation_acceptance_rate_percent="$(extract_summary_value "$quality_summary_txt" "parseability_generation_acceptance_rate_percent")"
quality_parse_full_passes="$(extract_summary_value "$quality_summary_txt" "parse_full_passes")"
quality_realistic_corpus_effective="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_effective")"
quality_realistic_cases_declared="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_cases_declared")"
quality_realistic_cases_executed="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_cases_executed")"
quality_realistic_expected_pass_total="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_expected_pass_total")"
quality_realistic_expected_fail_total="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_expected_fail_total")"
quality_realistic_observed_parse_pass_total="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_observed_parse_pass_total")"
quality_realistic_observed_parse_fail_total="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_observed_parse_fail_total")"
quality_realistic_expected_fail_parse_pass_total="$(extract_summary_value "$quality_summary_txt" "realistic_corpus_expected_fail_parse_pass_total")"
quality_total_errors="$(extract_summary_value "$quality_summary_txt" "total_errors")"

strict_promotion_recommendation="$(extract_summary_value "$strict_promotion_summary_txt" "recommendation")"
strict_promotion_note="$(extract_summary_value "$strict_promotion_summary_txt" "note")"
strict_promotion_eligible="$(extract_summary_value "$strict_promotion_summary_txt" "eligible_for_required_strict_mode")"
strict_promotion_trial_passed="$(extract_summary_value "$strict_promotion_summary_txt" "trial_passed")"
strict_promotion_trial_ratio_failures="$(extract_summary_value "$strict_promotion_summary_txt" "trial_ratio_failures")"
strict_promotion_trial_parity_failures="$(extract_summary_value "$strict_promotion_summary_txt" "trial_parity_failures")"
strict_promotion_trial_gate_failures="$(extract_summary_value "$strict_promotion_summary_txt" "trial_gate_failures")"
strict_promotion_trial_missing_ratio="$(extract_summary_value "$strict_promotion_summary_txt" "trial_missing_ratio")"
strict_promotion_primary_blocker="$(extract_summary_value "$strict_promotion_summary_txt" "primary_blocker")"
strict_promotion_observed_ratio_min="$(extract_summary_value "$strict_promotion_summary_txt" "observed_ratio_min")"
strict_promotion_observed_ratio_max="$(extract_summary_value "$strict_promotion_summary_txt" "observed_ratio_max")"
strict_promotion_observed_ratio_avg="$(extract_summary_value "$strict_promotion_summary_txt" "observed_ratio_avg")"

assert_equal "quality closed_loop_initial_status" "pass" "$quality_closed_loop_initial_status"
assert_equal "quality closed_loop_replay_status" "pass" "$quality_closed_loop_replay_status"
assert_equal "quality realistic_corpus_effective" "enabled" "$quality_realistic_corpus_effective"
assert_equal "quality realistic cases declared/executed" "$quality_realistic_cases_declared" "$quality_realistic_cases_executed"
assert_equal "quality realistic observed pass total" "$quality_realistic_expected_pass_total" "$quality_realistic_observed_parse_pass_total"
assert_equal "quality realistic observed fail total" "$quality_realistic_expected_fail_total" "$quality_realistic_observed_parse_fail_total"
assert_equal "quality expected_fail_parse_pass_total" "0" "$quality_realistic_expected_fail_parse_pass_total"
assert_equal "quality total_errors" "0" "$quality_total_errors"
assert_equal "quality parseability_generation_errors_total" "0" "$quality_parseability_generation_errors_total"
assert_equal "quality parseability_generation_empty_generations_total" "0" "$quality_parseability_generation_empty_generations_total"
assert_equal "quality shadow generation_errors_total" "0" "$quality_closed_loop_parseability_shadow_generation_errors_total"
assert_equal "quality shadow empty_generations_total" "0" "$quality_closed_loop_parseability_shadow_empty_generations_total"

quality_realistic_cases_status_failures="$(jq -r '[.cases[] | select(.status != "pass")] | length' "$quality_realistic_report_json")"
quality_realistic_cases_executed_json="$(jq -r '.totals.cases_executed' "$quality_realistic_report_json")"
quality_realistic_expected_pass_total_json="$(jq -r '.totals.expected_pass_total' "$quality_realistic_report_json")"
quality_realistic_expected_fail_total_json="$(jq -r '.totals.expected_fail_total' "$quality_realistic_report_json")"
quality_realistic_observed_pass_total_json="$(jq -r '.totals.observed_parse_pass_total' "$quality_realistic_report_json")"
quality_realistic_observed_fail_total_json="$(jq -r '.totals.observed_parse_fail_total' "$quality_realistic_report_json")"
quality_realistic_expected_fail_parse_pass_total_json="$(jq -r '.totals.expected_fail_parse_pass_total' "$quality_realistic_report_json")"
quality_parseability_requested_total_json="$(jq -r '.totals.requested_total' "$quality_parseability_report_json")"
quality_parseability_attempts_total_json="$(jq -r '.totals.attempts_total' "$quality_parseability_report_json")"
quality_parseability_accepted_total_json="$(jq -r '.totals.accepted_total' "$quality_parseability_report_json")"
quality_parseability_rejected_total_json="$(jq -r '.totals.rejected_total' "$quality_parseability_report_json")"
quality_parseability_parser_rejections_total_json="$(jq -r '.totals.parser_rejections_total' "$quality_parseability_report_json")"
quality_parseability_generation_errors_total_json="$(jq -r '.totals.generation_errors_total' "$quality_parseability_report_json")"
quality_parseability_empty_generations_total_json="$(jq -r '.totals.empty_generations_total' "$quality_parseability_report_json")"
quality_parseability_acceptance_rate_percent_json="$(jq -r '.totals.acceptance_rate_percent' "$quality_parseability_report_json")"

assert_equal "quality realistic cases_executed json parity" "$quality_realistic_cases_executed" "$quality_realistic_cases_executed_json"
assert_equal "quality realistic expected_pass_total json parity" "$quality_realistic_expected_pass_total" "$quality_realistic_expected_pass_total_json"
assert_equal "quality realistic expected_fail_total json parity" "$quality_realistic_expected_fail_total" "$quality_realistic_expected_fail_total_json"
assert_equal "quality realistic observed_parse_pass_total json parity" "$quality_realistic_observed_parse_pass_total" "$quality_realistic_observed_pass_total_json"
assert_equal "quality realistic observed_parse_fail_total json parity" "$quality_realistic_observed_parse_fail_total" "$quality_realistic_observed_fail_total_json"
assert_equal "quality realistic expected_fail_parse_pass_total json parity" "$quality_realistic_expected_fail_parse_pass_total" "$quality_realistic_expected_fail_parse_pass_total_json"
assert_equal "quality realistic case status failures" "0" "$quality_realistic_cases_status_failures"
assert_equal "quality parseability requested_total json parity" "$quality_parseability_generation_requested_total" "$quality_parseability_requested_total_json"
assert_equal "quality parseability attempts_total json parity" "$quality_parseability_generation_attempts_total" "$quality_parseability_attempts_total_json"
assert_equal "quality parseability accepted_total json parity" "$quality_parseability_generation_accepted_total" "$quality_parseability_accepted_total_json"
assert_equal "quality parseability rejected_total json parity" "$quality_parseability_generation_rejected_total" "$quality_parseability_rejected_total_json"
assert_equal "quality parseability parser_rejections_total json parity" "$quality_parseability_generation_parser_rejections_total" "$quality_parseability_parser_rejections_total_json"
assert_equal "quality parseability generation_errors_total json parity" "$quality_parseability_generation_errors_total" "$quality_parseability_generation_errors_total_json"
assert_equal "quality parseability empty_generations_total json parity" "$quality_parseability_generation_empty_generations_total" "$quality_parseability_empty_generations_total_json"
assert_equal "quality parseability acceptance_rate_percent json parity" "$quality_parseability_generation_acceptance_rate_percent" "$quality_parseability_acceptance_rate_percent_json"

strict_promotion_status_json="$(jq -r '.status' "$strict_promotion_report_json")"
strict_promotion_recommendation_json="$(jq -r '.recommendation' "$strict_promotion_report_json")"
strict_promotion_note_json="$(jq -r '.note' "$strict_promotion_report_json")"
strict_promotion_eligible_json="$(jq -r '.eligibility.eligible_for_required_strict_mode | if . then 1 else 0 end' "$strict_promotion_report_json")"
strict_promotion_trial_passed_json="$(jq -r '.totals.trial_passed' "$strict_promotion_report_json")"
strict_promotion_trial_ratio_failures_json="$(jq -r '.totals.trial_ratio_failures' "$strict_promotion_report_json")"
strict_promotion_trial_parity_failures_json="$(jq -r '.totals.trial_parity_failures' "$strict_promotion_report_json")"
strict_promotion_trial_gate_failures_json="$(jq -r '.totals.trial_gate_failures' "$strict_promotion_report_json")"
strict_promotion_trial_missing_ratio_json="$(jq -r '.totals.trial_missing_ratio' "$strict_promotion_report_json")"
strict_promotion_primary_blocker_json="$(jq -r '.blockers.primary_blocker' "$strict_promotion_report_json")"
strict_promotion_failed_trial_count_json="$(jq -r '.blockers.failed_trial_count' "$strict_promotion_report_json")"
strict_promotion_blocker_breakdown_count_json="$(jq -r '.blockers.breakdown | length' "$strict_promotion_report_json")"
strict_promotion_observed_ratio_min_json="$(jq -r '.totals.observed_ratio_min' "$strict_promotion_report_json")"
strict_promotion_observed_ratio_max_json="$(jq -r '.totals.observed_ratio_max' "$strict_promotion_report_json")"
strict_promotion_observed_ratio_avg_json="$(jq -r '.totals.observed_ratio_avg' "$strict_promotion_report_json")"
strict_promotion_parseability_generation_attempts_total_json="$(jq -r '(.parseability_generation.observed.attempts_total // 0)' "$strict_promotion_report_json")"
strict_promotion_closed_loop_shadow_attempts_total_json="$(jq -r '(.closed_loop_parseability_shadow.observed.attempts_total // 0)' "$strict_promotion_report_json")"

assert_equal "strict promotion status" "completed" "$strict_promotion_status_json"
assert_equal "strict promotion recommendation" "enable_required_strict_mode" "$strict_promotion_recommendation"
assert_equal "strict promotion recommendation json parity" "$strict_promotion_recommendation" "$strict_promotion_recommendation_json"
assert_equal "strict promotion note json parity" "$strict_promotion_note" "$strict_promotion_note_json"
assert_equal "strict promotion eligible_for_required_strict_mode" "1" "$strict_promotion_eligible"
assert_equal "strict promotion eligible json parity" "$strict_promotion_eligible" "$strict_promotion_eligible_json"
assert_equal "strict promotion trial_passed json parity" "$strict_promotion_trial_passed" "$strict_promotion_trial_passed_json"
assert_equal "strict promotion trial_ratio_failures" "0" "$strict_promotion_trial_ratio_failures"
assert_equal "strict promotion trial_ratio_failures json parity" "$strict_promotion_trial_ratio_failures" "$strict_promotion_trial_ratio_failures_json"
assert_equal "strict promotion trial_parity_failures" "0" "$strict_promotion_trial_parity_failures"
assert_equal "strict promotion trial_parity_failures json parity" "$strict_promotion_trial_parity_failures" "$strict_promotion_trial_parity_failures_json"
assert_equal "strict promotion trial_gate_failures" "0" "$strict_promotion_trial_gate_failures"
assert_equal "strict promotion trial_gate_failures json parity" "$strict_promotion_trial_gate_failures" "$strict_promotion_trial_gate_failures_json"
assert_equal "strict promotion trial_missing_ratio" "0" "$strict_promotion_trial_missing_ratio"
assert_equal "strict promotion trial_missing_ratio json parity" "$strict_promotion_trial_missing_ratio" "$strict_promotion_trial_missing_ratio_json"
assert_equal "strict promotion primary_blocker" "none" "$strict_promotion_primary_blocker"
assert_equal "strict promotion primary_blocker json parity" "$strict_promotion_primary_blocker" "$strict_promotion_primary_blocker_json"
assert_equal "strict promotion failed_trial_count" "0" "$strict_promotion_failed_trial_count_json"
assert_equal "strict promotion blocker breakdown count" "0" "$strict_promotion_blocker_breakdown_count_json"
assert_equal "strict promotion observed_ratio_min json parity" "$strict_promotion_observed_ratio_min" "$strict_promotion_observed_ratio_min_json"
assert_equal "strict promotion observed_ratio_max json parity" "$strict_promotion_observed_ratio_max" "$strict_promotion_observed_ratio_max_json"
assert_equal "strict promotion observed_ratio_avg json parity" "$strict_promotion_observed_ratio_avg" "$strict_promotion_observed_ratio_avg_json"

{
    echo "VHDL Parser Family Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "quality_state_dir: $quality_state_dir"
    echo "quality_summary_txt: $quality_summary_txt"
    echo "quality_contract_file: $quality_contract_file"
    echo "quality_closed_loop_initial_status: $quality_closed_loop_initial_status"
    echo "quality_closed_loop_replay_status: $quality_closed_loop_replay_status"
    echo "quality_closed_loop_initial_targets: $quality_closed_loop_initial_targets"
    echo "quality_closed_loop_replay_targets: $quality_closed_loop_replay_targets"
    echo "quality_closed_loop_parseability_shadow_requested_total: $quality_closed_loop_parseability_shadow_requested_total"
    echo "quality_closed_loop_parseability_shadow_attempts_total: $quality_closed_loop_parseability_shadow_attempts_total"
    echo "quality_closed_loop_parseability_shadow_accepted_total: $quality_closed_loop_parseability_shadow_accepted_total"
    echo "quality_closed_loop_parseability_shadow_rejected_total: $quality_closed_loop_parseability_shadow_rejected_total"
    echo "quality_closed_loop_parseability_shadow_parser_rejections_total: $quality_closed_loop_parseability_shadow_parser_rejections_total"
    echo "quality_closed_loop_parseability_shadow_acceptance_rate_percent: $quality_closed_loop_parseability_shadow_acceptance_rate_percent"
    echo "quality_parseability_generation_requested_total: $quality_parseability_generation_requested_total"
    echo "quality_parseability_generation_attempts_total: $quality_parseability_generation_attempts_total"
    echo "quality_parseability_generation_accepted_total: $quality_parseability_generation_accepted_total"
    echo "quality_parseability_generation_rejected_total: $quality_parseability_generation_rejected_total"
    echo "quality_parseability_generation_parser_rejections_total: $quality_parseability_generation_parser_rejections_total"
    echo "quality_parseability_generation_acceptance_rate_percent: $quality_parseability_generation_acceptance_rate_percent"
    echo "quality_parse_full_passes: $quality_parse_full_passes"
    echo "quality_realistic_cases_executed: $quality_realistic_cases_executed"
    echo "quality_realistic_expected_pass_total: $quality_realistic_expected_pass_total"
    echo "quality_realistic_expected_fail_total: $quality_realistic_expected_fail_total"
    echo "quality_realistic_observed_parse_pass_total: $quality_realistic_observed_parse_pass_total"
    echo "quality_realistic_observed_parse_fail_total: $quality_realistic_observed_parse_fail_total"
    echo "strict_promotion_state_dir: $strict_promotion_state_dir"
    echo "strict_promotion_summary_txt: $strict_promotion_summary_txt"
    echo "strict_promotion_recommendation: $strict_promotion_recommendation"
    echo "strict_promotion_eligible_for_required_strict_mode: $strict_promotion_eligible"
    echo "strict_promotion_primary_blocker: $strict_promotion_primary_blocker"
    echo "strict_promotion_trial_passed: $strict_promotion_trial_passed"
    echo "strict_promotion_observed_ratio_min: $strict_promotion_observed_ratio_min"
    echo "strict_promotion_observed_ratio_max: $strict_promotion_observed_ratio_max"
    echo "strict_promotion_observed_ratio_avg: $strict_promotion_observed_ratio_avg"
    echo "strict_promotion_parseability_generation_attempts_total: $strict_promotion_parseability_generation_attempts_total_json"
    echo "strict_promotion_closed_loop_shadow_attempts_total: $strict_promotion_closed_loop_shadow_attempts_total_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "vhdl_parser_family_contract_gate" \
    --argjson version 1 \
    --arg state_dir "$STATE_DIR" \
    --arg generated_at_utc "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --arg quality_state_dir "$quality_state_dir" \
    --arg quality_summary_txt "$quality_summary_txt" \
    --arg quality_realistic_report_json "$quality_realistic_report_json" \
    --arg quality_parseability_report_json "$quality_parseability_report_json" \
    --arg quality_contract_file "$quality_contract_file" \
    --arg quality_closed_loop_initial_status "$quality_closed_loop_initial_status" \
    --arg quality_closed_loop_replay_status "$quality_closed_loop_replay_status" \
    --argjson quality_closed_loop_initial_targets "$quality_closed_loop_initial_targets" \
    --argjson quality_closed_loop_replay_targets "$quality_closed_loop_replay_targets" \
    --argjson quality_closed_loop_parseability_shadow_requested_total "$quality_closed_loop_parseability_shadow_requested_total" \
    --argjson quality_closed_loop_parseability_shadow_attempts_total "$quality_closed_loop_parseability_shadow_attempts_total" \
    --argjson quality_closed_loop_parseability_shadow_accepted_total "$quality_closed_loop_parseability_shadow_accepted_total" \
    --argjson quality_closed_loop_parseability_shadow_rejected_total "$quality_closed_loop_parseability_shadow_rejected_total" \
    --argjson quality_closed_loop_parseability_shadow_parser_rejections_total "$quality_closed_loop_parseability_shadow_parser_rejections_total" \
    --argjson quality_closed_loop_parseability_shadow_generation_errors_total "$quality_closed_loop_parseability_shadow_generation_errors_total" \
    --argjson quality_closed_loop_parseability_shadow_empty_generations_total "$quality_closed_loop_parseability_shadow_empty_generations_total" \
    --arg quality_closed_loop_parseability_shadow_acceptance_rate_percent "$quality_closed_loop_parseability_shadow_acceptance_rate_percent" \
    --argjson quality_parseability_generation_requested_total "$quality_parseability_generation_requested_total" \
    --argjson quality_parseability_generation_attempts_total "$quality_parseability_generation_attempts_total" \
    --argjson quality_parseability_generation_accepted_total "$quality_parseability_generation_accepted_total" \
    --argjson quality_parseability_generation_rejected_total "$quality_parseability_generation_rejected_total" \
    --argjson quality_parseability_generation_parser_rejections_total "$quality_parseability_generation_parser_rejections_total" \
    --argjson quality_parseability_generation_errors_total "$quality_parseability_generation_errors_total" \
    --argjson quality_parseability_generation_empty_generations_total "$quality_parseability_generation_empty_generations_total" \
    --arg quality_parseability_generation_acceptance_rate_percent "$quality_parseability_generation_acceptance_rate_percent" \
    --arg quality_parse_full_passes "$quality_parse_full_passes" \
    --argjson quality_realistic_cases_executed "$quality_realistic_cases_executed" \
    --argjson quality_realistic_expected_pass_total "$quality_realistic_expected_pass_total" \
    --argjson quality_realistic_expected_fail_total "$quality_realistic_expected_fail_total" \
    --argjson quality_realistic_observed_parse_pass_total "$quality_realistic_observed_parse_pass_total" \
    --argjson quality_realistic_observed_parse_fail_total "$quality_realistic_observed_parse_fail_total" \
    --arg strict_promotion_state_dir "$strict_promotion_state_dir" \
    --arg strict_promotion_summary_txt "$strict_promotion_summary_txt" \
    --arg strict_promotion_report_json "$strict_promotion_report_json" \
    --arg strict_promotion_recommendation "$strict_promotion_recommendation" \
    --argjson strict_promotion_eligible_for_required_strict_mode "$strict_promotion_eligible" \
    --arg strict_promotion_primary_blocker "$strict_promotion_primary_blocker" \
    --argjson strict_promotion_trial_passed "$strict_promotion_trial_passed" \
    --arg strict_promotion_observed_ratio_min "$strict_promotion_observed_ratio_min" \
    --arg strict_promotion_observed_ratio_max "$strict_promotion_observed_ratio_max" \
    --arg strict_promotion_observed_ratio_avg "$strict_promotion_observed_ratio_avg" \
    --argjson strict_promotion_parseability_generation_attempts_total "$strict_promotion_parseability_generation_attempts_total_json" \
    --argjson strict_promotion_closed_loop_shadow_attempts_total "$strict_promotion_closed_loop_shadow_attempts_total_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      proof_surfaces: {
        quality_state_dir: $quality_state_dir,
        quality_summary_txt: $quality_summary_txt,
        quality_realistic_report_json: $quality_realistic_report_json,
        quality_parseability_report_json: $quality_parseability_report_json,
        strict_promotion_state_dir: $strict_promotion_state_dir,
        strict_promotion_summary_txt: $strict_promotion_summary_txt,
        strict_promotion_report_json: $strict_promotion_report_json
      },
      metrics: {
        quality_contract_file: $quality_contract_file,
        quality_closed_loop_initial_status: $quality_closed_loop_initial_status,
        quality_closed_loop_replay_status: $quality_closed_loop_replay_status,
        quality_closed_loop_initial_targets: $quality_closed_loop_initial_targets,
        quality_closed_loop_replay_targets: $quality_closed_loop_replay_targets,
        quality_closed_loop_parseability_shadow_requested_total: $quality_closed_loop_parseability_shadow_requested_total,
        quality_closed_loop_parseability_shadow_attempts_total: $quality_closed_loop_parseability_shadow_attempts_total,
        quality_closed_loop_parseability_shadow_accepted_total: $quality_closed_loop_parseability_shadow_accepted_total,
        quality_closed_loop_parseability_shadow_rejected_total: $quality_closed_loop_parseability_shadow_rejected_total,
        quality_closed_loop_parseability_shadow_parser_rejections_total: $quality_closed_loop_parseability_shadow_parser_rejections_total,
        quality_closed_loop_parseability_shadow_generation_errors_total: $quality_closed_loop_parseability_shadow_generation_errors_total,
        quality_closed_loop_parseability_shadow_empty_generations_total: $quality_closed_loop_parseability_shadow_empty_generations_total,
        quality_closed_loop_parseability_shadow_acceptance_rate_percent: $quality_closed_loop_parseability_shadow_acceptance_rate_percent,
        quality_parseability_generation_requested_total: $quality_parseability_generation_requested_total,
        quality_parseability_generation_attempts_total: $quality_parseability_generation_attempts_total,
        quality_parseability_generation_accepted_total: $quality_parseability_generation_accepted_total,
        quality_parseability_generation_rejected_total: $quality_parseability_generation_rejected_total,
        quality_parseability_generation_parser_rejections_total: $quality_parseability_generation_parser_rejections_total,
        quality_parseability_generation_errors_total: $quality_parseability_generation_errors_total,
        quality_parseability_generation_empty_generations_total: $quality_parseability_generation_empty_generations_total,
        quality_parseability_generation_acceptance_rate_percent: $quality_parseability_generation_acceptance_rate_percent,
        quality_parse_full_passes: $quality_parse_full_passes,
        quality_realistic_cases_executed: $quality_realistic_cases_executed,
        quality_realistic_expected_pass_total: $quality_realistic_expected_pass_total,
        quality_realistic_expected_fail_total: $quality_realistic_expected_fail_total,
        quality_realistic_observed_parse_pass_total: $quality_realistic_observed_parse_pass_total,
        quality_realistic_observed_parse_fail_total: $quality_realistic_observed_parse_fail_total,
        strict_promotion_recommendation: $strict_promotion_recommendation,
        strict_promotion_eligible_for_required_strict_mode: $strict_promotion_eligible_for_required_strict_mode,
        strict_promotion_primary_blocker: $strict_promotion_primary_blocker,
        strict_promotion_trial_passed: $strict_promotion_trial_passed,
        strict_promotion_observed_ratio_min: $strict_promotion_observed_ratio_min,
        strict_promotion_observed_ratio_max: $strict_promotion_observed_ratio_max,
        strict_promotion_observed_ratio_avg: $strict_promotion_observed_ratio_avg,
        strict_promotion_parseability_generation_attempts_total: $strict_promotion_parseability_generation_attempts_total,
        strict_promotion_closed_loop_shadow_attempts_total: $strict_promotion_closed_loop_shadow_attempts_total
      }
    }' >"$SUMMARY_JSON"

echo "✅ VHDL parser-family contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
