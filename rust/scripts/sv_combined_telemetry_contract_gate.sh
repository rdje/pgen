#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SOTA_EXIT_GATE_SCRIPT="$RUST_DIR/scripts/sota_exit_gate.sh"
SV_CONTRACT_FILE="${PGEN_SV_COMBINED_TELEMETRY_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json}"
SOTA_POLICY_ENV_FILE="${PGEN_SV_COMBINED_TELEMETRY_SOTA_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_combined_telemetry_lightweight_v0.env}"
EXISTING_SOTA_EXIT_STATE_DIR="${PGEN_SV_COMBINED_TELEMETRY_EXISTING_SOTA_EXIT_STATE_DIR:-}"

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

run_logged_with_env_file() {
    local label="$1"
    local env_file="$2"
    shift 2
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        set -a
        # shellcheck disable=SC1090
        source "$env_file"
        set +a
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

extract_summary_value() {
    local path="$1"
    local key="$2"
    awk -v key="$key" 'index($0, key ": ") == 1 { print substr($0, length(key) + 3); found = 1 } END { if (!found) exit 1 }' "$path"
}

assert_equal() {
    local label="$1"
    local expected="$2"
    local observed="$3"
    if [[ "$expected" != "$observed" ]]; then
        echo "error: ${label} mismatch (expected '${expected}', observed '${observed}')" >&2
        exit 1
    fi
}

require_file "$SOTA_EXIT_GATE_SCRIPT"
require_file "$SV_CONTRACT_FILE"
require_file "$SOTA_POLICY_ENV_FILE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

sota_state_dir="$WORK_DIR/sota_exit_gate"
if [[ -n "$EXISTING_SOTA_EXIT_STATE_DIR" ]]; then
    sota_state_dir="$EXISTING_SOTA_EXIT_STATE_DIR"
else
    run_logged_with_env_file "sv_combined_sota_exit_gate" "$SOTA_POLICY_ENV_FILE" \
        env PGEN_SOTA_EXIT_STATE_DIR="$sota_state_dir" \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$SV_CONTRACT_FILE" \
        "$SOTA_EXIT_GATE_SCRIPT"
fi

sota_summary_txt="$sota_state_dir/summary.txt"
sv_parser_aggregate_summary_txt="$sota_state_dir/work/sv_parser_aggregate_contract_gate/summary.txt"
sv_preprocessor_aggregate_summary_txt="$sota_state_dir/work/sv_preprocessor_aggregate_contract_gate/summary.txt"
sv_failure_summary_txt="$sota_state_dir/work/sv_failure_context_contract_gate/summary.txt"
sv_roundtrip_summary_txt="$sota_state_dir/work/sv_roundtrip_contract_gate/summary.txt"
sv_preprocessor_reachability_summary_txt="$sota_state_dir/work/sv_preprocessor_reachability_closure_gate/summary.txt"

require_nonempty_file "$sota_summary_txt"
require_nonempty_file "$sv_parser_aggregate_summary_txt"
require_nonempty_file "$sv_preprocessor_aggregate_summary_txt"
require_nonempty_file "$sv_failure_summary_txt"
require_nonempty_file "$sv_roundtrip_summary_txt"
require_nonempty_file "$sv_preprocessor_reachability_summary_txt"

assert_equal \
    "main SV aggregate summary path" \
    "$sv_parser_aggregate_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_stimuli_quality_aggregate_contract_summary_txt")"
assert_equal \
    "main SV failure-context summary path" \
    "$sv_failure_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_failure_context_contract_summary_txt")"
assert_equal \
    "main SV roundtrip summary path" \
    "$sv_roundtrip_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_contract_summary_txt")"
assert_equal \
    "SV preprocessor failure-context summary path" \
    "$sv_failure_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_failure_context_contract_summary_txt")"
assert_equal \
    "SV preprocessor roundtrip summary path" \
    "$sv_roundtrip_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_contract_summary_txt")"
assert_equal \
    "SV preprocessor aggregate summary path" \
    "$sv_preprocessor_aggregate_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_quality_aggregate_contract_summary_txt")"
assert_equal \
    "SV preprocessor reachability-closure summary path" \
    "$sv_preprocessor_reachability_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_closure_summary_txt")"

sv_replay_gap_target_triage_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_triage_json")"
sv_replay_gap_target_triage_txt="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_triage_txt")"
sv_replay_gap_target_unique_rules="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_unique_rules")"
sv_replay_gap_target_unique_reasons="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_unique_reasons")"
sv_replay_gap_target_unique_dependencies="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_unique_dependencies")"
sv_replay_gap_target_primary_target_type="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_target_type")"
sv_replay_gap_target_primary_target_type_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_target_type_count")"
sv_replay_gap_target_primary_reason="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_reason")"
sv_replay_gap_target_primary_reason_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_reason_count")"
sv_replay_gap_target_primary_rule="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_rule")"
sv_replay_gap_target_primary_rule_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_rule_count")"
sv_replay_gap_target_primary_dependency="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_dependency")"
sv_replay_gap_target_primary_dependency_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "replay_gap_target_primary_dependency_count")"
sv_generation_contract_file="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_contract_file")"
sv_generation_report_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_report_json")"
sv_generation_parser_rejections_total="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_parser_rejections_total")"
sv_generation_counterexamples_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexamples_count")"
sv_generation_counterexample_triage_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_triage_json")"
sv_generation_counterexample_triage_txt="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_triage_txt")"
sv_generation_counterexample_unique_shrunk_samples="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_unique_shrunk_samples")"
sv_generation_counterexample_primary_stage="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_stage")"
sv_generation_counterexample_primary_stage_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_stage_count")"
sv_generation_counterexample_primary_shrunk_sample="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_shrunk_sample")"
sv_generation_counterexample_primary_shrunk_sample_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_shrunk_sample_count")"
sv_generation_counterexample_primary_parser_error="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_parser_error")"
sv_generation_counterexample_primary_parser_error_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_parser_error_count")"
sv_generation_counterexample_primary_failure_location="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_location")"
sv_generation_counterexample_primary_failure_location_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_location_count")"
sv_generation_counterexample_primary_failure_line_excerpt_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_line_excerpt_json")"
sv_generation_counterexample_primary_failure_line_excerpt_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_line_excerpt_count")"
sv_generation_counterexample_primary_failure_context_excerpt_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_context_excerpt_json")"
sv_generation_counterexample_primary_failure_context_excerpt_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_primary_failure_context_excerpt_count")"
sv_generation_counterexample_unique_failure_locations="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_unique_failure_locations")"
sv_generation_counterexample_unique_failure_line_excerpts="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_unique_failure_line_excerpts")"
sv_generation_counterexample_unique_failure_context_excerpts="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "generation_counterexample_unique_failure_context_excerpts")"
sv_shadow_contract_file="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_contract_file")"
sv_shadow_report_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_report_json")"
sv_shadow_parser_rejections_total="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_parser_rejections_total")"
sv_shadow_counterexamples_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexamples_count")"
sv_shadow_counterexamples_captured_total="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexamples_captured_total")"
sv_shadow_counterexample_triage_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_triage_json")"
sv_shadow_counterexample_triage_txt="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_triage_txt")"
sv_shadow_counterexample_unique_shrunk_samples="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_unique_shrunk_samples")"
sv_shadow_counterexample_primary_stage="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_stage")"
sv_shadow_counterexample_primary_stage_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_stage_count")"
sv_shadow_counterexample_primary_shrunk_sample="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_shrunk_sample")"
sv_shadow_counterexample_primary_shrunk_sample_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_shrunk_sample_count")"
sv_shadow_counterexample_primary_parser_error="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_parser_error")"
sv_shadow_counterexample_primary_parser_error_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_parser_error_count")"
sv_shadow_counterexample_primary_failure_location="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_location")"
sv_shadow_counterexample_primary_failure_location_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_location_count")"
sv_shadow_counterexample_primary_failure_line_excerpt_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_line_excerpt_json")"
sv_shadow_counterexample_primary_failure_line_excerpt_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_line_excerpt_count")"
sv_shadow_counterexample_primary_failure_context_excerpt_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_context_excerpt_json")"
sv_shadow_counterexample_primary_failure_context_excerpt_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_primary_failure_context_excerpt_count")"
sv_shadow_counterexample_unique_failure_locations="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_unique_failure_locations")"
sv_shadow_counterexample_unique_failure_line_excerpts="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_unique_failure_line_excerpts")"
sv_shadow_counterexample_unique_failure_context_excerpts="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "shadow_counterexample_unique_failure_context_excerpts")"
sv_failure_generation_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_generation_failure_context_excerpts")"
sv_failure_shadow_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_shadow_failure_context_excerpts")"
sv_roundtrip_initial_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_targets")"
sv_roundtrip_replay_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_targets")"
sv_roundtrip_initial_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_covered_reachable_rules")"
sv_roundtrip_replay_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_covered_reachable_rules")"
sv_roundtrip_initial_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_covered_reachable_branches")"
sv_roundtrip_replay_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_covered_reachable_branches")"

svpp_failure_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_preprocessor_failure_context_excerpts")"
svpp_counterexample_triage_json="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_triage_json")"
svpp_counterexample_triage_txt="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_triage_txt")"
svpp_counterexample_unique_shrunk_samples="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_shrunk_samples")"
svpp_counterexample_primary_stage="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_stage")"
svpp_counterexample_primary_stage_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_stage_count")"
svpp_counterexample_primary_shrunk_sample="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_shrunk_sample")"
svpp_counterexample_primary_shrunk_sample_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_shrunk_sample_count")"
svpp_counterexample_primary_parser_error="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_parser_error")"
svpp_counterexample_primary_parser_error_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_parser_error_count")"
svpp_counterexample_primary_failure_location="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_location")"
svpp_counterexample_primary_failure_location_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_location_count")"
svpp_counterexample_primary_failure_line_excerpt_json="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_line_excerpt_json")"
svpp_counterexample_primary_failure_line_excerpt_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_line_excerpt_count")"
svpp_counterexample_primary_failure_context_excerpt_json="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_context_excerpt_json")"
svpp_counterexample_primary_failure_context_excerpt_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_context_excerpt_count")"
svpp_counterexample_unique_failure_locations="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_locations")"
svpp_counterexample_unique_failure_line_excerpts="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_line_excerpts")"
svpp_counterexample_unique_failure_context_excerpts="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_context_excerpts")"
svpp_stage0_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage0_targets")"
svpp_stage1_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage1_targets")"
svpp_final_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_final_targets")"
svpp_stage4_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage4_targets")"
svpp_stage0_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage0_covered_reachable_rules")"
svpp_stage1_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage1_covered_reachable_rules")"
svpp_stage4_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage4_covered_reachable_rules")"
svpp_stage0_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage0_covered_reachable_branches")"
svpp_stage1_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage1_covered_reachable_branches")"
svpp_stage4_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches")"
svpp_reachability_stage3_targets="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_targets")"
svpp_reachability_stage4_targets="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_targets")"
svpp_reachability_stage3_rules="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_covered_reachable_rules")"
svpp_reachability_stage4_rules="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_covered_reachable_rules")"
svpp_reachability_stage3_branches="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_covered_reachable_branches")"
svpp_reachability_stage4_branches="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_covered_reachable_branches")"
svpp_reachability_parseability_rejected="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "parseability_rejected")"
svpp_reachability_parser_rejections="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "parser_rejections")"

assert_equal \
    "main SV replay-gap triage json path" \
    "$sv_replay_gap_target_triage_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_triage_json")"
assert_equal \
    "main SV replay-gap triage txt path" \
    "$sv_replay_gap_target_triage_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_triage_txt")"
assert_equal \
    "main SV replay-gap unique rules" \
    "$sv_replay_gap_target_unique_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_unique_rules")"
assert_equal \
    "main SV replay-gap unique reasons" \
    "$sv_replay_gap_target_unique_reasons" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_unique_reasons")"
assert_equal \
    "main SV replay-gap unique dependencies" \
    "$sv_replay_gap_target_unique_dependencies" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_unique_dependencies")"
assert_equal \
    "main SV replay-gap primary target type" \
    "$sv_replay_gap_target_primary_target_type" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_target_type")"
assert_equal \
    "main SV replay-gap primary target type count" \
    "$sv_replay_gap_target_primary_target_type_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_target_type_count")"
assert_equal \
    "main SV replay-gap primary reason" \
    "$sv_replay_gap_target_primary_reason" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_reason")"
assert_equal \
    "main SV replay-gap primary reason count" \
    "$sv_replay_gap_target_primary_reason_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_reason_count")"
assert_equal \
    "main SV replay-gap primary rule" \
    "$sv_replay_gap_target_primary_rule" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_rule")"
assert_equal \
    "main SV replay-gap primary rule count" \
    "$sv_replay_gap_target_primary_rule_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_rule_count")"
assert_equal \
    "main SV replay-gap primary dependency" \
    "$sv_replay_gap_target_primary_dependency" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_dependency")"
assert_equal \
    "main SV replay-gap primary dependency count" \
    "$sv_replay_gap_target_primary_dependency_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_target_primary_dependency_count")"
assert_equal \
    "main SV generation contract file" \
    "$sv_generation_contract_file" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_contract_file")"
assert_equal \
    "main SV generation report json path" \
    "$sv_generation_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_report_json")"
assert_equal \
    "main SV generation parser rejections total" \
    "$sv_generation_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_parser_rejections_total")"
assert_equal \
    "main SV generation counterexamples count" \
    "$sv_generation_counterexamples_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexamples_count")"
assert_equal \
    "main SV generation triage json path" \
    "$sv_generation_counterexample_triage_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_triage_json")"
assert_equal \
    "main SV generation triage txt path" \
    "$sv_generation_counterexample_triage_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_triage_txt")"
assert_equal \
    "main SV generation unique shrunk samples" \
    "$sv_generation_counterexample_unique_shrunk_samples" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_unique_shrunk_samples")"
assert_equal \
    "main SV generation primary stage" \
    "$sv_generation_counterexample_primary_stage" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_stage")"
assert_equal \
    "main SV generation primary stage count" \
    "$sv_generation_counterexample_primary_stage_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_stage_count")"
assert_equal \
    "main SV generation primary shrunk sample" \
    "$sv_generation_counterexample_primary_shrunk_sample" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_shrunk_sample")"
assert_equal \
    "main SV generation primary shrunk sample count" \
    "$sv_generation_counterexample_primary_shrunk_sample_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_shrunk_sample_count")"
assert_equal \
    "main SV generation primary parser error" \
    "$sv_generation_counterexample_primary_parser_error" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_parser_error")"
assert_equal \
    "main SV generation primary parser error count" \
    "$sv_generation_counterexample_primary_parser_error_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_parser_error_count")"
assert_equal \
    "main SV generation primary failure location" \
    "$sv_generation_counterexample_primary_failure_location" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_location")"
assert_equal \
    "main SV generation primary failure location count" \
    "$sv_generation_counterexample_primary_failure_location_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_location_count")"
assert_equal \
    "main SV generation primary failure line excerpt json" \
    "$sv_generation_counterexample_primary_failure_line_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_line_excerpt_json")"
assert_equal \
    "main SV generation primary failure line excerpt count" \
    "$sv_generation_counterexample_primary_failure_line_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_line_excerpt_count")"
assert_equal \
    "main SV generation primary failure context excerpt json" \
    "$sv_generation_counterexample_primary_failure_context_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_context_excerpt_json")"
assert_equal \
    "main SV generation primary failure context excerpt count" \
    "$sv_generation_counterexample_primary_failure_context_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_primary_failure_context_excerpt_count")"
assert_equal \
    "main SV generation unique failure locations" \
    "$sv_generation_counterexample_unique_failure_locations" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_unique_failure_locations")"
assert_equal \
    "main SV generation unique failure line excerpts" \
    "$sv_generation_counterexample_unique_failure_line_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_unique_failure_line_excerpts")"
assert_equal \
    "main SV generation unique failure context excerpts" \
    "$sv_generation_counterexample_unique_failure_context_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_generation_counterexample_unique_failure_context_excerpts")"
assert_equal \
    "main SV shadow contract file" \
    "$sv_shadow_contract_file" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_contract_file")"
assert_equal \
    "main SV shadow report json path" \
    "$sv_shadow_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_report_json")"
assert_equal \
    "main SV shadow parser rejections total" \
    "$sv_shadow_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_parser_rejections_total")"
assert_equal \
    "main SV shadow counterexamples count" \
    "$sv_shadow_counterexamples_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexamples_count")"
assert_equal \
    "main SV shadow counterexamples captured total" \
    "$sv_shadow_counterexamples_captured_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexamples_captured_total")"
assert_equal \
    "main SV shadow triage json path" \
    "$sv_shadow_counterexample_triage_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_triage_json")"
assert_equal \
    "main SV shadow triage txt path" \
    "$sv_shadow_counterexample_triage_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_triage_txt")"
assert_equal \
    "main SV shadow unique shrunk samples" \
    "$sv_shadow_counterexample_unique_shrunk_samples" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_unique_shrunk_samples")"
assert_equal \
    "main SV shadow primary stage" \
    "$sv_shadow_counterexample_primary_stage" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_stage")"
assert_equal \
    "main SV shadow primary stage count" \
    "$sv_shadow_counterexample_primary_stage_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_stage_count")"
assert_equal \
    "main SV shadow primary shrunk sample" \
    "$sv_shadow_counterexample_primary_shrunk_sample" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_shrunk_sample")"
assert_equal \
    "main SV shadow primary shrunk sample count" \
    "$sv_shadow_counterexample_primary_shrunk_sample_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_shrunk_sample_count")"
assert_equal \
    "main SV shadow primary parser error" \
    "$sv_shadow_counterexample_primary_parser_error" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_parser_error")"
assert_equal \
    "main SV shadow primary parser error count" \
    "$sv_shadow_counterexample_primary_parser_error_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_parser_error_count")"
assert_equal \
    "main SV shadow primary failure location" \
    "$sv_shadow_counterexample_primary_failure_location" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_location")"
assert_equal \
    "main SV shadow primary failure location count" \
    "$sv_shadow_counterexample_primary_failure_location_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_location_count")"
assert_equal \
    "main SV shadow primary failure line excerpt json" \
    "$sv_shadow_counterexample_primary_failure_line_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_line_excerpt_json")"
assert_equal \
    "main SV shadow primary failure line excerpt count" \
    "$sv_shadow_counterexample_primary_failure_line_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_line_excerpt_count")"
assert_equal \
    "main SV shadow primary failure context excerpt json" \
    "$sv_shadow_counterexample_primary_failure_context_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_context_excerpt_json")"
assert_equal \
    "main SV shadow primary failure context excerpt count" \
    "$sv_shadow_counterexample_primary_failure_context_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_primary_failure_context_excerpt_count")"
assert_equal \
    "main SV shadow unique failure locations" \
    "$sv_shadow_counterexample_unique_failure_locations" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_unique_failure_locations")"
assert_equal \
    "main SV shadow unique failure line excerpts" \
    "$sv_shadow_counterexample_unique_failure_line_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_unique_failure_line_excerpts")"
assert_equal \
    "main SV shadow unique failure context excerpts" \
    "$sv_shadow_counterexample_unique_failure_context_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_shadow_counterexample_unique_failure_context_excerpts")"
assert_equal \
    "main SV generation failure-context excerpts" \
    "$sv_failure_generation_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_failure_context_generation_excerpts")"
assert_equal \
    "main SV shadow failure-context excerpts" \
    "$sv_failure_shadow_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_failure_context_shadow_excerpts")"
assert_equal \
    "main SV roundtrip initial targets" \
    "$sv_roundtrip_initial_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_targets")"
assert_equal \
    "main SV roundtrip replay targets" \
    "$sv_roundtrip_replay_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_targets")"
assert_equal \
    "main SV roundtrip initial reachable rules" \
    "$sv_roundtrip_initial_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_covered_reachable_rules")"
assert_equal \
    "main SV roundtrip replay reachable rules" \
    "$sv_roundtrip_replay_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_covered_reachable_rules")"
assert_equal \
    "main SV roundtrip initial reachable branches" \
    "$sv_roundtrip_initial_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_covered_reachable_branches")"
assert_equal \
    "main SV roundtrip replay reachable branches" \
    "$sv_roundtrip_replay_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_covered_reachable_branches")"

assert_equal \
    "SV preprocessor failure-context excerpts" \
    "$svpp_failure_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_failure_context_excerpts")"
assert_equal \
    "SV preprocessor counterexample triage json path" \
    "$svpp_counterexample_triage_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_triage_json")"
assert_equal \
    "SV preprocessor counterexample triage txt path" \
    "$svpp_counterexample_triage_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_triage_txt")"
assert_equal \
    "SV preprocessor counterexample unique shrunk samples" \
    "$svpp_counterexample_unique_shrunk_samples" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_unique_shrunk_samples")"
assert_equal \
    "SV preprocessor counterexample primary stage" \
    "$svpp_counterexample_primary_stage" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_stage")"
assert_equal \
    "SV preprocessor counterexample primary stage count" \
    "$svpp_counterexample_primary_stage_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_stage_count")"
assert_equal \
    "SV preprocessor counterexample primary shrunk sample" \
    "$svpp_counterexample_primary_shrunk_sample" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_shrunk_sample")"
assert_equal \
    "SV preprocessor counterexample primary shrunk sample count" \
    "$svpp_counterexample_primary_shrunk_sample_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_shrunk_sample_count")"
assert_equal \
    "SV preprocessor counterexample primary parser error" \
    "$svpp_counterexample_primary_parser_error" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_parser_error")"
assert_equal \
    "SV preprocessor counterexample primary parser error count" \
    "$svpp_counterexample_primary_parser_error_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_parser_error_count")"
assert_equal \
    "SV preprocessor counterexample primary failure location" \
    "$svpp_counterexample_primary_failure_location" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_location")"
assert_equal \
    "SV preprocessor counterexample primary failure location count" \
    "$svpp_counterexample_primary_failure_location_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_location_count")"
assert_equal \
    "SV preprocessor counterexample primary failure line excerpt json" \
    "$svpp_counterexample_primary_failure_line_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_line_excerpt_json")"
assert_equal \
    "SV preprocessor counterexample primary failure line excerpt count" \
    "$svpp_counterexample_primary_failure_line_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_line_excerpt_count")"
assert_equal \
    "SV preprocessor counterexample primary failure context excerpt json" \
    "$svpp_counterexample_primary_failure_context_excerpt_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_context_excerpt_json")"
assert_equal \
    "SV preprocessor counterexample primary failure context excerpt count" \
    "$svpp_counterexample_primary_failure_context_excerpt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_context_excerpt_count")"
assert_equal \
    "SV preprocessor counterexample unique failure locations" \
    "$svpp_counterexample_unique_failure_locations" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_unique_failure_locations")"
assert_equal \
    "SV preprocessor counterexample unique failure line excerpts" \
    "$svpp_counterexample_unique_failure_line_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_unique_failure_line_excerpts")"
assert_equal \
    "SV preprocessor counterexample unique failure context excerpts" \
    "$svpp_counterexample_unique_failure_context_excerpts" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_unique_failure_context_excerpts")"
assert_equal \
    "SV preprocessor roundtrip stage0 targets" \
    "$svpp_stage0_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_targets")"
assert_equal \
    "SV preprocessor roundtrip stage1 targets" \
    "$svpp_stage1_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_targets")"
assert_equal \
    "SV preprocessor roundtrip final targets" \
    "$svpp_final_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_final_targets")"
assert_equal \
    "SV preprocessor roundtrip stage4 targets" \
    "$svpp_stage4_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_targets")"
assert_equal \
    "SV preprocessor roundtrip stage0 reachable rules" \
    "$svpp_stage0_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_covered_reachable_rules")"
assert_equal \
    "SV preprocessor roundtrip stage1 reachable rules" \
    "$svpp_stage1_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_covered_reachable_rules")"
assert_equal \
    "SV preprocessor roundtrip stage4 reachable rules" \
    "$svpp_stage4_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_covered_reachable_rules")"
assert_equal \
    "SV preprocessor roundtrip stage0 reachable branches" \
    "$svpp_stage0_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_covered_reachable_branches")"
assert_equal \
    "SV preprocessor roundtrip stage1 reachable branches" \
    "$svpp_stage1_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_covered_reachable_branches")"
assert_equal \
    "SV preprocessor roundtrip stage4 reachable branches" \
    "$svpp_stage4_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_covered_reachable_branches")"
assert_equal \
    "SV preprocessor reachability stage3 targets" \
    "$svpp_reachability_stage3_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage3_targets")"
assert_equal \
    "SV preprocessor reachability stage4 targets" \
    "$svpp_reachability_stage4_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage4_targets")"
assert_equal \
    "SV preprocessor reachability stage3 reachable rules" \
    "$svpp_reachability_stage3_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage3_covered_reachable_rules")"
assert_equal \
    "SV preprocessor reachability stage4 reachable rules" \
    "$svpp_reachability_stage4_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage4_covered_reachable_rules")"
assert_equal \
    "SV preprocessor reachability stage3 reachable branches" \
    "$svpp_reachability_stage3_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage3_covered_reachable_branches")"
assert_equal \
    "SV preprocessor reachability stage4 reachable branches" \
    "$svpp_reachability_stage4_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_stage4_covered_reachable_branches")"
assert_equal \
    "SV preprocessor reachability parseability rejected" \
    "$svpp_reachability_parseability_rejected" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_parseability_rejected")"
assert_equal \
    "SV preprocessor reachability parser rejections" \
    "$svpp_reachability_parser_rejections" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_parser_rejections")"

{
    echo "SV Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
    echo "existing_sota_exit_state_dir: ${EXISTING_SOTA_EXIT_STATE_DIR:-<unset>}"
    echo "sota_exit_state_dir: $sota_state_dir"
    echo "sota_exit_summary_txt: $sota_summary_txt"
    echo "sv_stimuli_quality_aggregate_contract_summary_txt: $sv_parser_aggregate_summary_txt"
    echo "sv_preprocessor_quality_aggregate_contract_summary_txt: $sv_preprocessor_aggregate_summary_txt"
    echo "sv_failure_context_contract_summary_txt: $sv_failure_summary_txt"
    echo "sv_roundtrip_contract_summary_txt: $sv_roundtrip_summary_txt"
    echo "sv_preprocessor_reachability_closure_summary_txt: $sv_preprocessor_reachability_summary_txt"
    echo "sv_replay_gap_target_triage_json: $sv_replay_gap_target_triage_json"
    echo "sv_replay_gap_target_triage_txt: $sv_replay_gap_target_triage_txt"
    echo "sv_replay_gap_target_unique_rules: $sv_replay_gap_target_unique_rules"
    echo "sv_replay_gap_target_unique_reasons: $sv_replay_gap_target_unique_reasons"
    echo "sv_replay_gap_target_unique_dependencies: $sv_replay_gap_target_unique_dependencies"
    echo "sv_replay_gap_target_primary_target_type: $sv_replay_gap_target_primary_target_type"
    echo "sv_replay_gap_target_primary_target_type_count: $sv_replay_gap_target_primary_target_type_count"
    echo "sv_replay_gap_target_primary_reason: $sv_replay_gap_target_primary_reason"
    echo "sv_replay_gap_target_primary_reason_count: $sv_replay_gap_target_primary_reason_count"
    echo "sv_replay_gap_target_primary_rule: $sv_replay_gap_target_primary_rule"
    echo "sv_replay_gap_target_primary_rule_count: $sv_replay_gap_target_primary_rule_count"
    echo "sv_replay_gap_target_primary_dependency: $sv_replay_gap_target_primary_dependency"
    echo "sv_replay_gap_target_primary_dependency_count: $sv_replay_gap_target_primary_dependency_count"
    echo "sv_generation_contract_file: $sv_generation_contract_file"
    echo "sv_generation_report_json: $sv_generation_report_json"
    echo "sv_generation_parser_rejections_total: $sv_generation_parser_rejections_total"
    echo "sv_generation_counterexamples_count: $sv_generation_counterexamples_count"
    echo "sv_generation_counterexample_triage_json: $sv_generation_counterexample_triage_json"
    echo "sv_generation_counterexample_triage_txt: $sv_generation_counterexample_triage_txt"
    echo "sv_generation_counterexample_unique_shrunk_samples: $sv_generation_counterexample_unique_shrunk_samples"
    echo "sv_generation_counterexample_primary_stage: $sv_generation_counterexample_primary_stage"
    echo "sv_generation_counterexample_primary_stage_count: $sv_generation_counterexample_primary_stage_count"
    echo "sv_generation_counterexample_primary_shrunk_sample: $sv_generation_counterexample_primary_shrunk_sample"
    echo "sv_generation_counterexample_primary_shrunk_sample_count: $sv_generation_counterexample_primary_shrunk_sample_count"
    echo "sv_generation_counterexample_primary_parser_error: $sv_generation_counterexample_primary_parser_error"
    echo "sv_generation_counterexample_primary_parser_error_count: $sv_generation_counterexample_primary_parser_error_count"
    echo "sv_generation_counterexample_primary_failure_location: $sv_generation_counterexample_primary_failure_location"
    echo "sv_generation_counterexample_primary_failure_location_count: $sv_generation_counterexample_primary_failure_location_count"
    echo "sv_generation_counterexample_primary_failure_line_excerpt_json: $sv_generation_counterexample_primary_failure_line_excerpt_json"
    echo "sv_generation_counterexample_primary_failure_line_excerpt_count: $sv_generation_counterexample_primary_failure_line_excerpt_count"
    echo "sv_generation_counterexample_primary_failure_context_excerpt_json: $sv_generation_counterexample_primary_failure_context_excerpt_json"
    echo "sv_generation_counterexample_primary_failure_context_excerpt_count: $sv_generation_counterexample_primary_failure_context_excerpt_count"
    echo "sv_generation_counterexample_unique_failure_locations: $sv_generation_counterexample_unique_failure_locations"
    echo "sv_generation_counterexample_unique_failure_line_excerpts: $sv_generation_counterexample_unique_failure_line_excerpts"
    echo "sv_generation_counterexample_unique_failure_context_excerpts: $sv_generation_counterexample_unique_failure_context_excerpts"
    echo "sv_shadow_contract_file: $sv_shadow_contract_file"
    echo "sv_shadow_report_json: $sv_shadow_report_json"
    echo "sv_shadow_parser_rejections_total: $sv_shadow_parser_rejections_total"
    echo "sv_shadow_counterexamples_count: $sv_shadow_counterexamples_count"
    echo "sv_shadow_counterexamples_captured_total: $sv_shadow_counterexamples_captured_total"
    echo "sv_shadow_counterexample_triage_json: $sv_shadow_counterexample_triage_json"
    echo "sv_shadow_counterexample_triage_txt: $sv_shadow_counterexample_triage_txt"
    echo "sv_shadow_counterexample_unique_shrunk_samples: $sv_shadow_counterexample_unique_shrunk_samples"
    echo "sv_shadow_counterexample_primary_stage: $sv_shadow_counterexample_primary_stage"
    echo "sv_shadow_counterexample_primary_stage_count: $sv_shadow_counterexample_primary_stage_count"
    echo "sv_shadow_counterexample_primary_shrunk_sample: $sv_shadow_counterexample_primary_shrunk_sample"
    echo "sv_shadow_counterexample_primary_shrunk_sample_count: $sv_shadow_counterexample_primary_shrunk_sample_count"
    echo "sv_shadow_counterexample_primary_parser_error: $sv_shadow_counterexample_primary_parser_error"
    echo "sv_shadow_counterexample_primary_parser_error_count: $sv_shadow_counterexample_primary_parser_error_count"
    echo "sv_shadow_counterexample_primary_failure_location: $sv_shadow_counterexample_primary_failure_location"
    echo "sv_shadow_counterexample_primary_failure_location_count: $sv_shadow_counterexample_primary_failure_location_count"
    echo "sv_shadow_counterexample_primary_failure_line_excerpt_json: $sv_shadow_counterexample_primary_failure_line_excerpt_json"
    echo "sv_shadow_counterexample_primary_failure_line_excerpt_count: $sv_shadow_counterexample_primary_failure_line_excerpt_count"
    echo "sv_shadow_counterexample_primary_failure_context_excerpt_json: $sv_shadow_counterexample_primary_failure_context_excerpt_json"
    echo "sv_shadow_counterexample_primary_failure_context_excerpt_count: $sv_shadow_counterexample_primary_failure_context_excerpt_count"
    echo "sv_shadow_counterexample_unique_failure_locations: $sv_shadow_counterexample_unique_failure_locations"
    echo "sv_shadow_counterexample_unique_failure_line_excerpts: $sv_shadow_counterexample_unique_failure_line_excerpts"
    echo "sv_shadow_counterexample_unique_failure_context_excerpts: $sv_shadow_counterexample_unique_failure_context_excerpts"
    echo "sv_failure_context_generation_excerpts: $sv_failure_generation_excerpts"
    echo "sv_failure_context_shadow_excerpts: $sv_failure_shadow_excerpts"
    echo "sv_roundtrip_initial_targets: $sv_roundtrip_initial_targets"
    echo "sv_roundtrip_replay_targets: $sv_roundtrip_replay_targets"
    echo "sv_roundtrip_initial_covered_reachable_rules: $sv_roundtrip_initial_rules"
    echo "sv_roundtrip_replay_covered_reachable_rules: $sv_roundtrip_replay_rules"
    echo "sv_roundtrip_initial_covered_reachable_branches: $sv_roundtrip_initial_branches"
    echo "sv_roundtrip_replay_covered_reachable_branches: $sv_roundtrip_replay_branches"
    echo "sv_preprocessor_failure_context_excerpts: $svpp_failure_excerpts"
    echo "sv_preprocessor_counterexample_triage_json: $svpp_counterexample_triage_json"
    echo "sv_preprocessor_counterexample_triage_txt: $svpp_counterexample_triage_txt"
    echo "sv_preprocessor_counterexample_unique_shrunk_samples: $svpp_counterexample_unique_shrunk_samples"
    echo "sv_preprocessor_counterexample_primary_stage: $svpp_counterexample_primary_stage"
    echo "sv_preprocessor_counterexample_primary_stage_count: $svpp_counterexample_primary_stage_count"
    echo "sv_preprocessor_counterexample_primary_shrunk_sample: $svpp_counterexample_primary_shrunk_sample"
    echo "sv_preprocessor_counterexample_primary_shrunk_sample_count: $svpp_counterexample_primary_shrunk_sample_count"
    echo "sv_preprocessor_counterexample_primary_parser_error: $svpp_counterexample_primary_parser_error"
    echo "sv_preprocessor_counterexample_primary_parser_error_count: $svpp_counterexample_primary_parser_error_count"
    echo "sv_preprocessor_counterexample_primary_failure_location: $svpp_counterexample_primary_failure_location"
    echo "sv_preprocessor_counterexample_primary_failure_location_count: $svpp_counterexample_primary_failure_location_count"
    echo "sv_preprocessor_counterexample_primary_failure_line_excerpt_json: $svpp_counterexample_primary_failure_line_excerpt_json"
    echo "sv_preprocessor_counterexample_primary_failure_line_excerpt_count: $svpp_counterexample_primary_failure_line_excerpt_count"
    echo "sv_preprocessor_counterexample_primary_failure_context_excerpt_json: $svpp_counterexample_primary_failure_context_excerpt_json"
    echo "sv_preprocessor_counterexample_primary_failure_context_excerpt_count: $svpp_counterexample_primary_failure_context_excerpt_count"
    echo "sv_preprocessor_counterexample_unique_failure_locations: $svpp_counterexample_unique_failure_locations"
    echo "sv_preprocessor_counterexample_unique_failure_line_excerpts: $svpp_counterexample_unique_failure_line_excerpts"
    echo "sv_preprocessor_counterexample_unique_failure_context_excerpts: $svpp_counterexample_unique_failure_context_excerpts"
    echo "sv_preprocessor_roundtrip_stage0_targets: $svpp_stage0_targets"
    echo "sv_preprocessor_roundtrip_stage1_targets: $svpp_stage1_targets"
    echo "sv_preprocessor_roundtrip_final_targets: $svpp_final_targets"
    echo "sv_preprocessor_roundtrip_stage4_targets: $svpp_stage4_targets"
    echo "sv_preprocessor_roundtrip_stage0_covered_reachable_rules: $svpp_stage0_rules"
    echo "sv_preprocessor_roundtrip_stage1_covered_reachable_rules: $svpp_stage1_rules"
    echo "sv_preprocessor_roundtrip_stage4_covered_reachable_rules: $svpp_stage4_rules"
    echo "sv_preprocessor_roundtrip_stage0_covered_reachable_branches: $svpp_stage0_branches"
    echo "sv_preprocessor_roundtrip_stage1_covered_reachable_branches: $svpp_stage1_branches"
    echo "sv_preprocessor_roundtrip_stage4_covered_reachable_branches: $svpp_stage4_branches"
    echo "sv_preprocessor_reachability_stage3_targets: $svpp_reachability_stage3_targets"
    echo "sv_preprocessor_reachability_stage4_targets: $svpp_reachability_stage4_targets"
    echo "sv_preprocessor_reachability_stage3_covered_reachable_rules: $svpp_reachability_stage3_rules"
    echo "sv_preprocessor_reachability_stage4_covered_reachable_rules: $svpp_reachability_stage4_rules"
    echo "sv_preprocessor_reachability_stage3_covered_reachable_branches: $svpp_reachability_stage3_branches"
    echo "sv_preprocessor_reachability_stage4_covered_reachable_branches: $svpp_reachability_stage4_branches"
    echo "sv_preprocessor_reachability_parseability_rejected: $svpp_reachability_parseability_rejected"
    echo "sv_preprocessor_reachability_parser_rejections: $svpp_reachability_parser_rejections"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
