#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SOTA_EXIT_GATE_SCRIPT="$RUST_DIR/scripts/sota_exit_gate.sh"
SV_CONTRACT_FILE="${PGEN_SV_COMBINED_TELEMETRY_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json}"
SOTA_POLICY_ENV_FILE="${PGEN_SV_COMBINED_TELEMETRY_SOTA_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_combined_telemetry_lightweight_v0.env}"
EXISTING_SOTA_EXIT_STATE_DIR="${PGEN_SV_COMBINED_TELEMETRY_EXISTING_SOTA_EXIT_STATE_DIR:-}"

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

extract_summary_value_or_default() {
    local path="$1"
    local key="$2"
    local default_value="$3"
    awk -v key="$key" -v default_value="$default_value" '
        index($0, key ": ") == 1 {
            print substr($0, length(key) + 3)
            found = 1
        }
        END {
            if (!found) {
                print default_value
            }
        }
    ' "$path"
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
require_tool jq

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
sota_summary_json="$sota_state_dir/summary.json"
require_nonempty_file "$sota_summary_txt"
require_nonempty_file "$sota_summary_json"

sota_exit_gate_name="$(jq -r '.gate' "$sota_summary_json")"
sota_exit_gate_version="$(jq -r '.version' "$sota_summary_json")"
sota_exit_generated_at_utc="$(jq -r '.generated_at_utc' "$sota_summary_json")"
sota_exit_status="$(jq -r '.status' "$sota_summary_json")"
sota_exit_summary_txt_from_json="$(jq -r '.proof_surfaces.summary_txt' "$sota_summary_json")"
sota_exit_summary_csv_from_json="$(jq -r '.proof_surfaces.summary_csv' "$sota_summary_json")"
sota_exit_summary_json_from_json="$(jq -r '.proof_surfaces.summary_json' "$sota_summary_json")"
sota_exit_sv_failure_context_summary_json_from_json="$(jq -r '.proof_surfaces.sv_failure_context_contract_summary_json' "$sota_summary_json")"
sota_exit_sv_roundtrip_summary_json_from_json="$(jq -r '.proof_surfaces.sv_roundtrip_contract_summary_json' "$sota_summary_json")"
sota_exit_required_failures="$(jq -r '.counts.required_failures' "$sota_summary_json")"
sota_exit_informational_failures="$(jq -r '.counts.informational_failures' "$sota_summary_json")"
sota_exit_all_failures="$(jq -r '.counts.all_failures' "$sota_summary_json")"
sota_exit_sv_parser_family_status_summary_json_from_json="$(jq -r '.proof_surfaces.sv_parser_family_status_summary_json' "$sota_summary_json")"
sota_exit_sv_parser_family_status_contract_summary_json_from_json="$(jq -r '.proof_surfaces.sv_parser_family_status_contract_summary_json' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_state_dir_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_parser_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_summary_txt_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_parser_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_summary_json_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_parser_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_state_dir_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_preprocessor_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_summary_txt_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_preprocessor_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_summary_json_top_level="$(jq -r '.proof_surfaces.sv_family_status_systemverilog_preprocessor_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_state_dir_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_parser_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_summary_txt_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_parser_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_summary_json_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_parser_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_state_dir_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_txt_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_json_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_state_dir_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_txt_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_json_top_level="$(jq -r '.proof_surfaces.sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_state_dir="$(jq -r '.family_status.systemverilog.proof_surfaces.parser_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_summary_txt="$(jq -r '.family_status.systemverilog.proof_surfaces.parser_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_systemverilog_parser_aggregate_summary_json="$(jq -r '.family_status.systemverilog.proof_surfaces.parser_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_state_dir="$(jq -r '.family_status.systemverilog_preprocessor.proof_surfaces.aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_summary_txt="$(jq -r '.family_status.systemverilog_preprocessor.proof_surfaces.aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_systemverilog_preprocessor_aggregate_summary_json="$(jq -r '.family_status.systemverilog_preprocessor.proof_surfaces.aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_state_dir="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.parser_aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_summary_txt="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.parser_aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_parser_aggregate_summary_json="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.parser_aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_state_dir="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.semantic_scope_contract_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_txt="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.semantic_scope_contract_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_json="$(jq -r '.family_status_contract.systemverilog.proof_surfaces.semantic_scope_contract_summary_json' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_state_dir="$(jq -r '.family_status_contract.systemverilog_preprocessor.proof_surfaces.aggregate_state_dir' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_txt="$(jq -r '.family_status_contract.systemverilog_preprocessor.proof_surfaces.aggregate_summary_txt' "$sota_summary_json")"
sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_json="$(jq -r '.family_status_contract.systemverilog_preprocessor.proof_surfaces.aggregate_summary_json' "$sota_summary_json")"
sota_exit_sv_systemverilog_primary_unmet="$(jq -r '.family_status.systemverilog.primary_unmet_closure_criterion' "$sota_summary_json")"
sota_exit_sv_systemverilog_unmet_json="$(jq -cer '.family_status.systemverilog.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_sv_systemverilog_unmet_details_json="$(jq -cer '.family_status.systemverilog.unmet_closure_criteria_details' "$sota_summary_json")"
sota_exit_sv_systemverilog_primary_unmet_detail="$(jq -r '.family_status_contract.systemverilog.primary_unmet_detail_criterion' "$sota_summary_json")"
sota_exit_sv_systemverilog_unmet_detail_json="$(jq -cer '.family_status_contract.systemverilog.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_sv_systemverilog_unmet_detail_details_json="$(jq -cer '.family_status_contract.systemverilog.unmet_closure_criteria_details' "$sota_summary_json")"
sota_exit_svpp_primary_unmet="$(jq -r '.family_status.systemverilog_preprocessor.primary_unmet_closure_criterion' "$sota_summary_json")"
sota_exit_svpp_unmet_json="$(jq -cer '.family_status.systemverilog_preprocessor.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_svpp_unmet_details_json="$(jq -cer '.family_status.systemverilog_preprocessor.unmet_closure_criteria_details' "$sota_summary_json")"
sota_exit_svpp_primary_unmet_detail="$(jq -r '.family_status_contract.systemverilog_preprocessor.primary_unmet_detail_criterion' "$sota_summary_json")"
sota_exit_svpp_unmet_detail_json="$(jq -cer '.family_status_contract.systemverilog_preprocessor.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_svpp_unmet_detail_details_json="$(jq -cer '.family_status_contract.systemverilog_preprocessor.unmet_closure_criteria_details' "$sota_summary_json")"

assert_equal \
    "SOTA exit gate name" \
    "$(extract_summary_value "$sota_summary_txt" "gate")" \
    "$sota_exit_gate_name"
assert_equal \
    "SOTA exit gate version" \
    "$(extract_summary_value "$sota_summary_txt" "version")" \
    "$sota_exit_gate_version"
assert_equal \
    "SOTA exit generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "generated_at_utc")" \
    "$sota_exit_generated_at_utc"
assert_equal \
    "SOTA exit summary txt path from JSON" \
    "$sota_summary_txt" \
    "$sota_exit_summary_txt_from_json"
assert_equal \
    "SOTA exit summary csv path from JSON" \
    "$(extract_summary_value "$sota_summary_txt" "summary_csv")" \
    "$sota_exit_summary_csv_from_json"
assert_equal \
    "SOTA exit summary json self-path" \
    "$sota_summary_json" \
    "$sota_exit_summary_json_from_json"
assert_equal \
    "SOTA exit status" \
    "passed" \
    "$sota_exit_status"
assert_equal \
    "SOTA exit required failures" \
    "0" \
    "$sota_exit_required_failures"
assert_equal \
    "SOTA exit informational failures" \
    "$(extract_summary_value "$sota_summary_txt" "informational_failures")" \
    "$sota_exit_informational_failures"
assert_equal \
    "SOTA exit all failures" \
    "$(extract_summary_value "$sota_summary_txt" "all_failures")" \
    "$sota_exit_all_failures"
sv_parser_aggregate_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_stimuli_quality_aggregate_contract_summary_txt")"
sv_preprocessor_aggregate_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_quality_aggregate_contract_summary_txt")"
sv_failure_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_failure_context_contract_summary_txt")"
sv_failure_summary_json="$(extract_summary_value "$sota_summary_txt" "sv_failure_context_contract_summary_json")"
sv_roundtrip_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_contract_summary_txt")"
sv_roundtrip_summary_json="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_contract_summary_json")"
sv_preprocessor_reachability_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_reachability_closure_summary_txt")"
sv_parser_family_status_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_summary_txt")"
sv_parser_family_status_summary_json="$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_summary_json")"
sv_parser_family_status_contract_summary_txt="$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_summary_txt")"
sv_parser_family_status_contract_summary_json="$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_summary_json")"

require_nonempty_file "$sv_parser_aggregate_summary_txt"
require_nonempty_file "$sv_preprocessor_aggregate_summary_txt"
require_nonempty_file "$sv_failure_summary_txt"
require_nonempty_file "$sv_failure_summary_json"
require_nonempty_file "$sv_roundtrip_summary_txt"
require_nonempty_file "$sv_roundtrip_summary_json"
require_nonempty_file "$sv_preprocessor_reachability_summary_txt"
require_nonempty_file "$sv_parser_family_status_summary_txt"
require_nonempty_file "$sv_parser_family_status_summary_json"
require_nonempty_file "$sv_parser_family_status_contract_summary_txt"
require_nonempty_file "$sv_parser_family_status_contract_summary_json"

assert_equal \
    "SOTA exit SV parser-family status summary json path" \
    "$sv_parser_family_status_summary_json" \
    "$sota_exit_sv_parser_family_status_summary_json_from_json"
assert_equal \
    "SOTA exit SV parser-family status contract summary json path" \
    "$sv_parser_family_status_contract_summary_json" \
    "$sota_exit_sv_parser_family_status_contract_summary_json_from_json"
assert_equal \
    "SOTA exit SV failure-context contract summary json path" \
    "$sv_failure_summary_json" \
    "$sota_exit_sv_failure_context_summary_json_from_json"
assert_equal \
    "SOTA exit SV roundtrip contract summary json path" \
    "$sv_roundtrip_summary_json" \
    "$sota_exit_sv_roundtrip_summary_json_from_json"
assert_equal \
    "SOTA exit main parser aggregate state dir mirror" \
    "$sota_exit_sv_systemverilog_parser_aggregate_state_dir" \
    "$sota_exit_sv_systemverilog_parser_aggregate_state_dir_top_level"
assert_equal \
    "SOTA exit main parser aggregate summary txt mirror" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_txt" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_txt_top_level"
assert_equal \
    "SOTA exit main parser aggregate summary json mirror" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_json" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_json_top_level"
assert_equal \
    "SOTA exit preprocessor aggregate state dir mirror" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_state_dir" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_state_dir_top_level"
assert_equal \
    "SOTA exit preprocessor aggregate summary txt mirror" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_txt" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_txt_top_level"
assert_equal \
    "SOTA exit preprocessor aggregate summary json mirror" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_json" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_json_top_level"
assert_equal \
    "SOTA exit main contract parser aggregate state dir mirror" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_state_dir" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_state_dir_top_level"
assert_equal \
    "SOTA exit main contract parser aggregate summary txt mirror" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_txt_top_level"
assert_equal \
    "SOTA exit main contract parser aggregate summary json mirror" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_json" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_json_top_level"
assert_equal \
    "SOTA exit main contract semantic-scope state dir mirror" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_state_dir" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_state_dir_top_level"
assert_equal \
    "SOTA exit main contract semantic-scope summary txt mirror" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_txt_top_level"
assert_equal \
    "SOTA exit main contract semantic-scope summary json mirror" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_json" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_json_top_level"
assert_equal \
    "SOTA exit preprocessor contract aggregate state dir mirror" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_state_dir" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_state_dir_top_level"
assert_equal \
    "SOTA exit preprocessor contract aggregate summary txt mirror" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_txt_top_level"
assert_equal \
    "SOTA exit preprocessor contract aggregate summary json mirror" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_json" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_json_top_level"

sv_failure_summary_available=0
sv_roundtrip_summary_available=0
if [[ -s "$sv_failure_summary_txt" && -s "$sv_failure_summary_json" ]]; then
    sv_failure_summary_available=1
fi
if [[ -s "$sv_roundtrip_summary_txt" && -s "$sv_roundtrip_summary_json" ]]; then
    sv_roundtrip_summary_available=1
fi

sv_failure_gate_name="$(jq -r '.gate' "$sv_failure_summary_json")"
sv_failure_gate_version="$(jq -r '.version' "$sv_failure_summary_json")"
sv_failure_generated_at_utc="$(jq -r '.generated_at_utc' "$sv_failure_summary_json")"
sv_failure_summary_txt_from_json="$(jq -r '.summary_txt' "$sv_failure_summary_json")"
sv_failure_summary_json_from_json="$(jq -r '.summary_json' "$sv_failure_summary_json")"

sv_roundtrip_gate_name="$(jq -r '.gate' "$sv_roundtrip_summary_json")"
sv_roundtrip_gate_version="$(jq -r '.version' "$sv_roundtrip_summary_json")"
sv_roundtrip_generated_at_utc="$(jq -r '.generated_at_utc' "$sv_roundtrip_summary_json")"
sv_roundtrip_summary_txt_from_json="$(jq -r '.summary_txt' "$sv_roundtrip_summary_json")"
sv_roundtrip_summary_json_from_json="$(jq -r '.summary_json' "$sv_roundtrip_summary_json")"

assert_equal \
    "SV failure-context summary txt self path" \
    "$sv_failure_summary_txt" \
    "$sv_failure_summary_txt_from_json"
assert_equal \
    "SV failure-context summary json self path" \
    "$sv_failure_summary_json" \
    "$sv_failure_summary_json_from_json"
assert_equal \
    "SV roundtrip summary txt self path" \
    "$sv_roundtrip_summary_txt" \
    "$sv_roundtrip_summary_txt_from_json"
assert_equal \
    "SV roundtrip summary json self path" \
    "$sv_roundtrip_summary_json" \
    "$sv_roundtrip_summary_json_from_json"

sv_parser_family_status_gate_name="$(jq -r '.gate' "$sv_parser_family_status_summary_json")"
sv_parser_family_status_gate_version="$(jq -r '.version' "$sv_parser_family_status_summary_json")"
sv_parser_family_status_generated_at_utc="$(jq -r '.generated_at_utc' "$sv_parser_family_status_summary_json")"
sv_parser_family_status_live_tracker_file="$(jq -r '.live_tracker_file' "$sv_parser_family_status_summary_json")"
sv_parser_family_status_status_rule_done="$(jq -r '.status_rule_done' "$sv_parser_family_status_summary_json")"

assert_equal \
    "SV parser-family status gate name" \
    "$sv_parser_family_status_gate_name" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_gate")"
assert_equal \
    "SV parser-family status gate version" \
    "$sv_parser_family_status_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_gate_version")"
assert_equal \
    "SV parser-family status generated_at_utc" \
    "$sv_parser_family_status_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_generated_at_utc")"
assert_equal \
    "SV parser-family status live tracker file" \
    "$sv_parser_family_status_live_tracker_file" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_live_tracker_file")"
assert_equal \
    "SV parser-family status Done rule" \
    "$sv_parser_family_status_status_rule_done" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_status_rule_done")"

sv_parser_family_status_contract_family_count="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "family_count")"
sv_parser_family_status_contract_systemverilog_tracker_alignment_ok="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_tracker_alignment_ok")"
sv_parser_family_status_contract_systemverilog_false_criteria_count="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_false_criteria_count")"
sv_parser_family_status_contract_systemverilog_unmet_details_count="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_unmet_details_count")"
sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_primary_unmet_detail_criterion")"
sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_unmet_closure_criteria_json")"
sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_unmet_closure_criteria_details_json")"
sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_tracker_alignment_ok")"
sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_false_criteria_count")"
sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_unmet_details_count")"
sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_primary_unmet_detail_criterion")"
sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_json")"
sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json="$(extract_summary_value "$sv_parser_family_status_contract_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_details_json")"
sv_family_status_contract_gate="$(jq -r '.gate' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_gate_version="$(jq -r '.version' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_generated_at_utc="$(jq -r '.generated_at_utc' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_family_status_state_dir="$(jq -r '.family_status_state_dir' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_family_status_summary_json="$(jq -r '.family_status_summary_json' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_family_status_summary_txt="$(jq -r '.family_status_summary_txt' "$sv_parser_family_status_contract_summary_json")"

assert_equal \
    "SV parser-family status contract family count" \
    "$sv_parser_family_status_contract_family_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_family_count")"
assert_equal \
    "SV family-status contract gate name" \
    "$sv_family_status_contract_gate" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_gate")"
assert_equal \
    "SV family-status contract gate version" \
    "$sv_family_status_contract_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_gate_version")"
assert_equal \
    "SV family-status contract generated_at_utc" \
    "$sv_family_status_contract_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_generated_at_utc")"
assert_equal \
    "SV family-status contract family-status state dir" \
    "$sv_family_status_contract_family_status_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_family_status_state_dir")"
assert_equal \
    "SV family-status contract family-status summary json" \
    "$sv_family_status_contract_family_status_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_family_status_summary_json")"
assert_equal \
    "SV family-status contract family-status summary txt" \
    "$sv_family_status_contract_family_status_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_family_status_summary_txt")"
assert_equal \
    "SV parser-family status contract main tracker alignment" \
    "$sv_parser_family_status_contract_systemverilog_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_tracker_alignment_ok")"
assert_equal \
    "SV parser-family status contract main false criteria count" \
    "$sv_parser_family_status_contract_systemverilog_false_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_false_criteria_count")"
assert_equal \
    "SV parser-family status contract main unmet details count" \
    "$sv_parser_family_status_contract_systemverilog_unmet_details_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_unmet_details_count")"
assert_equal \
    "SV parser-family status contract main primary unmet detail criterion" \
    "$sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion")"
assert_equal \
    "SV parser-family status contract main unmet criteria json" \
    "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json")"
assert_equal \
    "SV parser-family status contract main unmet criteria details json" \
    "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json")"
assert_equal \
    "SV parser-family status contract preprocessor tracker alignment" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok")"
assert_equal \
    "SV parser-family status contract preprocessor false criteria count" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count")"
assert_equal \
    "SV parser-family status contract preprocessor unmet details count" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count")"
assert_equal \
    "SV parser-family status contract preprocessor primary unmet detail criterion" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion")"
assert_equal \
    "SV parser-family status contract preprocessor unmet criteria json" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json")"
assert_equal \
    "SV parser-family status contract preprocessor unmet criteria details json" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json")"

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
sv_replay_gap_source_gap_json="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "source_gap_json")"
sv_base_contract_file="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "base_contract_file")"
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
sv_focused_initial_target_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_initial_target_count")"
sv_focused_replay_target_count="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_replay_target_count")"
sv_focused_initial_covered_reachable_rules="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_initial_covered_reachable_rules")"
sv_focused_replay_covered_reachable_rules="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_replay_covered_reachable_rules")"
sv_focused_initial_covered_reachable_branches="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_initial_covered_reachable_branches")"
sv_focused_replay_covered_reachable_branches="$(extract_summary_value "$sv_parser_aggregate_summary_txt" "focused_replay_covered_reachable_branches")"
if [[ "$sv_failure_summary_available" -eq 1 ]]; then
    sv_failure_generation_excerpts="$(jq -r '.metrics.systemverilog_generation_failure_context_excerpts' "$sv_failure_summary_json")"
    sv_failure_shadow_excerpts="$(jq -r '.metrics.systemverilog_shadow_failure_context_excerpts' "$sv_failure_summary_json")"
    svpp_failure_excerpts="$(jq -r '.metrics.systemverilog_preprocessor_failure_context_excerpts' "$sv_failure_summary_json")"
else
    sv_failure_generation_excerpts="$(extract_summary_value "$sota_summary_txt" "sv_failure_context_generation_excerpts")"
    sv_failure_shadow_excerpts="$(extract_summary_value "$sota_summary_txt" "sv_failure_context_shadow_excerpts")"
    svpp_failure_excerpts="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_failure_context_excerpts")"
fi
if [[ "$sv_roundtrip_summary_available" -eq 1 ]]; then
    sv_roundtrip_initial_targets="$(jq -r '.metrics.systemverilog_roundtrip_initial_targets' "$sv_roundtrip_summary_json")"
    sv_roundtrip_replay_targets="$(jq -r '.metrics.systemverilog_roundtrip_replay_targets' "$sv_roundtrip_summary_json")"
    sv_roundtrip_initial_rules="$(jq -r '.metrics.systemverilog_roundtrip_initial_covered_reachable_rules' "$sv_roundtrip_summary_json")"
    sv_roundtrip_replay_rules="$(jq -r '.metrics.systemverilog_roundtrip_replay_covered_reachable_rules' "$sv_roundtrip_summary_json")"
    sv_roundtrip_initial_branches="$(jq -r '.metrics.systemverilog_roundtrip_initial_covered_reachable_branches' "$sv_roundtrip_summary_json")"
    sv_roundtrip_replay_branches="$(jq -r '.metrics.systemverilog_roundtrip_replay_covered_reachable_branches' "$sv_roundtrip_summary_json")"
    svpp_stage0_targets="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage0_targets' "$sv_roundtrip_summary_json")"
    svpp_stage1_targets="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage1_targets' "$sv_roundtrip_summary_json")"
    svpp_final_targets="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_final_targets' "$sv_roundtrip_summary_json")"
    svpp_stage4_targets="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage4_targets' "$sv_roundtrip_summary_json")"
    svpp_stage0_rules="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage0_covered_reachable_rules' "$sv_roundtrip_summary_json")"
    svpp_stage1_rules="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage1_covered_reachable_rules' "$sv_roundtrip_summary_json")"
    svpp_stage4_rules="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage4_covered_reachable_rules' "$sv_roundtrip_summary_json")"
    svpp_stage0_branches="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage0_covered_reachable_branches' "$sv_roundtrip_summary_json")"
    svpp_stage1_branches="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage1_covered_reachable_branches' "$sv_roundtrip_summary_json")"
    svpp_stage4_branches="$(jq -r '.metrics.systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches' "$sv_roundtrip_summary_json")"
else
    sv_roundtrip_initial_targets="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_targets")"
    sv_roundtrip_replay_targets="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_targets")"
    sv_roundtrip_initial_rules="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_covered_reachable_rules")"
    sv_roundtrip_replay_rules="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_covered_reachable_rules")"
    sv_roundtrip_initial_branches="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_initial_covered_reachable_branches")"
    sv_roundtrip_replay_branches="$(extract_summary_value "$sota_summary_txt" "sv_roundtrip_replay_covered_reachable_branches")"
    svpp_stage0_targets="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_targets")"
    svpp_stage1_targets="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_targets")"
    svpp_final_targets="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_final_targets")"
    svpp_stage4_targets="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_targets")"
    svpp_stage0_rules="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_covered_reachable_rules")"
    svpp_stage1_rules="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_covered_reachable_rules")"
    svpp_stage4_rules="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_covered_reachable_rules")"
    svpp_stage0_branches="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage0_covered_reachable_branches")"
    svpp_stage1_branches="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage1_covered_reachable_branches")"
    svpp_stage4_branches="$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_roundtrip_stage4_covered_reachable_branches")"
fi

svpp_parseability_report_json="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_report_json")"
svpp_gap_stage3_json="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "gap_stage3_json")"
svpp_parseability_attempts_total="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_attempts_total")"
svpp_parseability_accepted_total="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_accepted_total")"
svpp_parseability_rejected_total="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_rejected_total")"
svpp_parseability_parser_rejections_total="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_parser_rejections_total")"
svpp_parseability_counterexamples_captured_total="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "parseability_counterexamples_captured_total")"
svpp_stage0_target_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage0_target_count")"
svpp_stage1_target_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage1_target_count")"
svpp_final_target_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "final_targets")"
svpp_stage4_target_count="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage4_target_count")"
svpp_stage0_covered_reachable_rules="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage0_covered_reachable_rules")"
svpp_stage1_covered_reachable_rules="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage1_covered_reachable_rules")"
svpp_covered_reachable_rules="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "covered_reachable_rules")"
svpp_stage4_covered_reachable_rules="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage4_covered_reachable_rules")"
svpp_stage0_covered_reachable_branches="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage0_covered_reachable_branches")"
svpp_stage1_covered_reachable_branches="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage1_covered_reachable_branches")"
svpp_covered_reachable_branches="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "covered_reachable_branches")"
svpp_stage4_covered_reachable_branches="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "stage4_covered_reachable_branches")"
svpp_fuzz_replay_accepted_cases="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "fuzz_replay_accepted_cases")"
svpp_fuzz_replay_rejected_cases="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "fuzz_replay_rejected_cases")"
svpp_fuzz_replay_parseability_counterexamples="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "fuzz_replay_parseability_counterexamples")"
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
svpp_counterexample_primary_failure_line_excerpt_json="$(extract_summary_value_or_default "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_line_excerpt_json" "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_line_excerpt_json")")"
svpp_counterexample_primary_failure_line_excerpt_count="$(extract_summary_value_or_default "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_line_excerpt_count" "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_line_excerpt_count")")"
svpp_counterexample_primary_failure_context_excerpt_json="$(extract_summary_value_or_default "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_context_excerpt_json" "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_context_excerpt_json")")"
svpp_counterexample_primary_failure_context_excerpt_count="$(extract_summary_value_or_default "$sv_preprocessor_aggregate_summary_txt" "counterexample_primary_failure_context_excerpt_count" "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_counterexample_primary_failure_context_excerpt_count")")"
svpp_counterexample_unique_failure_locations="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_locations")"
svpp_counterexample_unique_failure_line_excerpts="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_line_excerpts")"
svpp_counterexample_unique_failure_context_excerpts="$(extract_summary_value "$sv_preprocessor_aggregate_summary_txt" "counterexample_unique_failure_context_excerpts")"
svpp_reachability_stage3_targets="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_targets")"
svpp_reachability_stage4_targets="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_targets")"
svpp_reachability_stage3_rules="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_covered_reachable_rules")"
svpp_reachability_stage4_rules="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_covered_reachable_rules")"
svpp_reachability_stage3_branches="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage3_covered_reachable_branches")"
svpp_reachability_stage4_branches="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "stage4_covered_reachable_branches")"
svpp_reachability_parseability_rejected="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "parseability_rejected")"
svpp_reachability_parser_rejections="$(extract_summary_value "$sv_preprocessor_reachability_summary_txt" "parser_rejections")"
sv_family_status_systemverilog="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_status")"
sv_family_status_systemverilog_tracker_status="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_tracker_status")"
sv_family_status_systemverilog_tracker_alignment_ok="$(jq -r '.families[] | select(.family=="systemverilog") | .tracker_alignment_ok' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_unmet_closure_criteria_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_unmet_closure_criteria_count")"
sv_family_status_systemverilog_unmet_closure_criteria_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_unmet_closure_criteria_details_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria_details' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_primary_unmet_closure_criterion="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_primary_unmet_closure_criterion")"
sv_family_status_systemverilog_primary_unmet_closure_criterion="${sv_family_status_systemverilog_primary_unmet_closure_criterion:-<none>}"
sv_family_status_systemverilog_closure_criteria_total_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_closure_criteria_total_count")"
sv_family_status_systemverilog_closure_criteria_satisfied_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_closure_criteria_satisfied_count")"
sv_family_status_systemverilog_closure_criteria_unsatisfied_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_closure_criteria_unsatisfied_count")"
sv_family_status_systemverilog_syntax_closure_status="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_closure_status")"
sv_family_status_systemverilog_syntax_closure_failure_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_closure_failure_count")"
sv_family_status_systemverilog_syntax_defined_rule_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_defined_rule_count")"
sv_family_status_systemverilog_syntax_unresolved_rule_reference_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_unresolved_rule_reference_count")"
sv_family_status_systemverilog_syntax_unreachable_rules="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_unreachable_rules")"
sv_family_status_systemverilog_syntax_unreachable_branches="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_unreachable_branches")"
sv_family_status_systemverilog_syntax_target_debt_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_syntax_target_debt_count")"
sv_family_status_systemverilog_syntax_closure_gate_green="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.syntax_closure_gate_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_parser_aggregate_contract_green="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.parser_aggregate_contract_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_generation_parser_rejections_zero="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.generation_parser_rejections_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_replay_shadow_parser_rejections_zero="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.replay_shadow_parser_rejections_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_focused_replay_target_debt_zero="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.focused_replay_target_debt_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_generation_parser_rejections_total="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.generation_parser_rejections_total' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_replay_shadow_parser_rejections_total="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.replay_shadow_parser_rejections_total' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_focused_replay_target_count="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.focused_replay_target_count' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_focused_replay_covered_reachable_rules="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.focused_replay_covered_reachable_rules' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_focused_replay_covered_reachable_branches="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.focused_replay_covered_reachable_branches' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_replay_gap_target_primary_rule="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.replay_gap_target_primary_rule' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_case_count="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.semantic_scope_case_count' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_failed_count="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.semantic_scope_failed_count' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_syntax_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_syntax_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_syntax_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_summary_json' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_parser_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_parser_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_parser_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_json' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_contract_green="$(jq -r '.families[] | select(.family=="systemverilog") | .criteria.semantic_scope_contract_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_contract_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_contract_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_semantic_scope_contract_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_json' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_status")"
sv_family_status_systemverilog_preprocessor_tracker_status="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_tracker_status")"
sv_family_status_systemverilog_preprocessor_tracker_alignment_ok="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .tracker_alignment_ok' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_count")"
sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria_details' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_primary_unmet_closure_criterion")"
sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion="${sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion:-<none>}"
sv_family_status_systemverilog_preprocessor_closure_criteria_total_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_closure_criteria_total_count")"
sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_closure_criteria_satisfied_count")"
sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_closure_criteria_unsatisfied_count")"
sv_family_status_systemverilog_preprocessor_syntax_closure_status="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_closure_status")"
sv_family_status_systemverilog_preprocessor_syntax_closure_failure_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_closure_failure_count")"
sv_family_status_systemverilog_preprocessor_syntax_defined_rule_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_defined_rule_count")"
sv_family_status_systemverilog_preprocessor_syntax_unresolved_rule_reference_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_unresolved_rule_reference_count")"
sv_family_status_systemverilog_preprocessor_syntax_unreachable_rules="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_unreachable_rules")"
sv_family_status_systemverilog_preprocessor_syntax_unreachable_branches="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_unreachable_branches")"
sv_family_status_systemverilog_preprocessor_syntax_target_debt_count="$(extract_summary_value "$sv_parser_family_status_summary_txt" "systemverilog_preprocessor_syntax_target_debt_count")"
sv_family_status_systemverilog_preprocessor_syntax_closure_gate_green="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.syntax_closure_gate_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_aggregate_contract_green="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.aggregate_contract_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_closure_green="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_closure_green' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_parser_rejections_zero="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.parser_rejections_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_parseability_rejections_zero="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.parseability_rejections_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_targets_zero="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage3_targets_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_targets_zero="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage4_targets_zero' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_rules_full="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage3_rules_full' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_rules_full="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage4_rules_full' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_branches_full="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage3_branches_full' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_branches_full="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .criteria.reachability_stage4_branches_full' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.parseability_parser_rejections_total' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_parseability_rejected_total="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.parseability_rejected_total' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_final_targets="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.final_targets' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_covered_reachable_rules="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.covered_reachable_rules' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_covered_reachable_branches="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.covered_reachable_branches' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_counterexample_primary_stage="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.counterexample_primary_stage' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_targets="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage3_targets' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_targets="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage4_targets' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_rules="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage3_rules' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_rules="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage4_rules' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage3_branches="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage3_branches' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_stage4_branches="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.reachability_stage4_branches' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_summary_json' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_json' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.reachability_state_dir' "$sv_parser_family_status_summary_json")"
sv_family_status_systemverilog_preprocessor_reachability_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.reachability_summary_txt' "$sv_parser_family_status_summary_json")"
sv_family_status_contract_systemverilog_parser_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_state_dir' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_parser_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_txt' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_parser_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_json' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_state_dir' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_txt' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_json' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_state_dir' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_txt' "$sv_parser_family_status_contract_summary_json")"
sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_json' "$sv_parser_family_status_contract_summary_json")"

assert_equal \
    "SOTA exit main family primary unmet closure criterion" \
    "$sv_family_status_systemverilog_primary_unmet_closure_criterion" \
    "$sota_exit_sv_systemverilog_primary_unmet"
assert_equal \
    "SOTA exit main family unmet criteria json" \
    "$sv_family_status_systemverilog_unmet_closure_criteria_json" \
    "$sota_exit_sv_systemverilog_unmet_json"
assert_equal \
    "SOTA exit main family unmet criteria details json" \
    "$sv_family_status_systemverilog_unmet_closure_criteria_details_json" \
    "$sota_exit_sv_systemverilog_unmet_details_json"
assert_equal \
    "SOTA exit main family primary unmet detail criterion" \
    "$sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion" \
    "$sota_exit_sv_systemverilog_primary_unmet_detail"
assert_equal \
    "SOTA exit main family unmet detail criteria json" \
    "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json" \
    "$sota_exit_sv_systemverilog_unmet_detail_json"
assert_equal \
    "SOTA exit main family unmet detail criteria details json" \
    "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json" \
    "$sota_exit_sv_systemverilog_unmet_detail_details_json"
assert_equal \
    "SOTA exit preprocessor family primary unmet closure criterion" \
    "$sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion" \
    "$sota_exit_svpp_primary_unmet"
assert_equal \
    "SOTA exit preprocessor family unmet criteria json" \
    "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json" \
    "$sota_exit_svpp_unmet_json"
assert_equal \
    "SOTA exit preprocessor family unmet criteria details json" \
    "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    "$sota_exit_svpp_unmet_details_json"
assert_equal \
    "SOTA exit preprocessor family primary unmet detail criterion" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion" \
    "$sota_exit_svpp_primary_unmet_detail"
assert_equal \
    "SOTA exit preprocessor family unmet detail criteria json" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json" \
    "$sota_exit_svpp_unmet_detail_json"
assert_equal \
    "SOTA exit preprocessor family unmet detail criteria details json" \
    "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    "$sota_exit_svpp_unmet_detail_details_json"

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
    "main SV replay-gap source gap json path" \
    "$sv_replay_gap_source_gap_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_replay_gap_source_gap_json")"
assert_equal \
    "main SV base contract file" \
    "$sv_base_contract_file" \
    "$(extract_summary_value "$sota_summary_txt" "sv_base_contract_file")"
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
    "main SV focused initial target count" \
    "$sv_focused_initial_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_initial_target_count")"
assert_equal \
    "main SV focused replay target count" \
    "$sv_focused_replay_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_replay_target_count")"
assert_equal \
    "main SV focused initial reachable rules" \
    "$sv_focused_initial_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_initial_covered_reachable_rules")"
assert_equal \
    "main SV focused replay reachable rules" \
    "$sv_focused_replay_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_replay_covered_reachable_rules")"
assert_equal \
    "main SV focused initial reachable branches" \
    "$sv_focused_initial_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_initial_covered_reachable_branches")"
assert_equal \
    "main SV focused replay reachable branches" \
    "$sv_focused_replay_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_focused_replay_covered_reachable_branches")"
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
    "SV preprocessor parseability report json path" \
    "$svpp_parseability_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_report_json")"
assert_equal \
    "SV preprocessor stage3 gap json path" \
    "$svpp_gap_stage3_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_gap_stage3_json")"
assert_equal \
    "SV preprocessor parseability attempts total" \
    "$svpp_parseability_attempts_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_attempts_total")"
assert_equal \
    "SV preprocessor parseability accepted total" \
    "$svpp_parseability_accepted_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_accepted_total")"
assert_equal \
    "SV preprocessor parseability rejected total" \
    "$svpp_parseability_rejected_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_rejected_total")"
assert_equal \
    "SV preprocessor parseability parser rejections total" \
    "$svpp_parseability_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_parser_rejections_total")"
assert_equal \
    "SV preprocessor parseability counterexamples captured total" \
    "$svpp_parseability_counterexamples_captured_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_parseability_counterexamples_captured_total")"
assert_equal \
    "SV preprocessor stage0 target count" \
    "$svpp_stage0_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage0_target_count")"
assert_equal \
    "SV preprocessor stage1 target count" \
    "$svpp_stage1_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage1_target_count")"
assert_equal \
    "SV preprocessor final target count" \
    "$svpp_final_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_final_targets")"
assert_equal \
    "SV preprocessor stage4 target count" \
    "$svpp_stage4_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage4_target_count")"
assert_equal \
    "SV preprocessor stage0 covered reachable rules" \
    "$svpp_stage0_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage0_covered_reachable_rules")"
assert_equal \
    "SV preprocessor stage1 covered reachable rules" \
    "$svpp_stage1_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage1_covered_reachable_rules")"
assert_equal \
    "SV preprocessor covered reachable rules" \
    "$svpp_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_covered_reachable_rules")"
assert_equal \
    "SV preprocessor stage4 covered reachable rules" \
    "$svpp_stage4_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage4_covered_reachable_rules")"
assert_equal \
    "SV preprocessor stage0 covered reachable branches" \
    "$svpp_stage0_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage0_covered_reachable_branches")"
assert_equal \
    "SV preprocessor stage1 covered reachable branches" \
    "$svpp_stage1_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage1_covered_reachable_branches")"
assert_equal \
    "SV preprocessor covered reachable branches" \
    "$svpp_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_covered_reachable_branches")"
assert_equal \
    "SV preprocessor stage4 covered reachable branches" \
    "$svpp_stage4_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_stage4_covered_reachable_branches")"
assert_equal \
    "SV preprocessor fuzz replay accepted cases" \
    "$svpp_fuzz_replay_accepted_cases" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_fuzz_replay_accepted_cases")"
assert_equal \
    "SV preprocessor fuzz replay rejected cases" \
    "$svpp_fuzz_replay_rejected_cases" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_fuzz_replay_rejected_cases")"
assert_equal \
    "SV preprocessor fuzz replay parseability counterexamples" \
    "$svpp_fuzz_replay_parseability_counterexamples" \
    "$(extract_summary_value "$sota_summary_txt" "sv_preprocessor_fuzz_replay_parseability_counterexamples")"
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
assert_equal \
    "SV family-status main parser label" \
    "$sv_family_status_systemverilog" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog")"
assert_equal \
    "SV family-status main parser tracker status" \
    "$sv_family_status_systemverilog_tracker_status" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_tracker_status")"
assert_equal \
    "SV family-status main parser tracker alignment" \
    "$sv_family_status_systemverilog_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_tracker_alignment_ok")"
assert_equal \
    "SV family-status main parser unmet closure criteria count" \
    "$sv_family_status_systemverilog_unmet_closure_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_unmet_closure_criteria_count")"
assert_equal \
    "SV family-status main parser unmet closure criteria json" \
    "$sv_family_status_systemverilog_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_unmet_closure_criteria_json")"
assert_equal \
    "SV family-status main parser unmet closure criteria details json" \
    "$sv_family_status_systemverilog_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_unmet_closure_criteria_details_json")"
assert_equal \
    "SV family-status main parser closure criteria total count" \
    "$sv_family_status_systemverilog_closure_criteria_total_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_closure_criteria_total_count")"
assert_equal \
    "SV family-status main parser closure criteria satisfied count" \
    "$sv_family_status_systemverilog_closure_criteria_satisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_closure_criteria_satisfied_count")"
assert_equal \
    "SV family-status main parser closure criteria unsatisfied count" \
    "$sv_family_status_systemverilog_closure_criteria_unsatisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_closure_criteria_unsatisfied_count")"
assert_equal \
    "SV family-status main parser primary unmet closure criterion" \
    "$sv_family_status_systemverilog_primary_unmet_closure_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_primary_unmet_closure_criterion")"
assert_equal \
    "SV family-status main parser syntax-closure status" \
    "$sv_family_status_systemverilog_syntax_closure_status" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_status")"
assert_equal \
    "SV family-status main parser syntax-closure failure count" \
    "$sv_family_status_systemverilog_syntax_closure_failure_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_failure_count")"
assert_equal \
    "SV family-status main parser syntax defined rule count" \
    "$sv_family_status_systemverilog_syntax_defined_rule_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_defined_rule_count")"
assert_equal \
    "SV family-status main parser syntax unresolved rule reference count" \
    "$sv_family_status_systemverilog_syntax_unresolved_rule_reference_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_unresolved_rule_reference_count")"
assert_equal \
    "SV family-status main parser syntax unreachable rules" \
    "$sv_family_status_systemverilog_syntax_unreachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_unreachable_rules")"
assert_equal \
    "SV family-status main parser syntax unreachable branches" \
    "$sv_family_status_systemverilog_syntax_unreachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_unreachable_branches")"
assert_equal \
    "SV family-status main parser syntax target debt count" \
    "$sv_family_status_systemverilog_syntax_target_debt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_target_debt_count")"
assert_equal \
    "SV family-status main parser syntax-closure gate green" \
    "$sv_family_status_systemverilog_syntax_closure_gate_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_gate_green")"
assert_equal \
    "SV family-status main parser aggregate-contract green" \
    "$sv_family_status_systemverilog_parser_aggregate_contract_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_parser_aggregate_contract_green")"
assert_equal \
    "SV family-status main parser generation-rejections zero" \
    "$sv_family_status_systemverilog_generation_parser_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_generation_parser_rejections_zero")"
assert_equal \
    "SV family-status main parser replay-shadow rejections zero" \
    "$sv_family_status_systemverilog_replay_shadow_parser_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_replay_shadow_parser_rejections_zero")"
assert_equal \
    "SV family-status main parser focused replay debt zero" \
    "$sv_family_status_systemverilog_focused_replay_target_debt_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_focused_replay_target_debt_zero")"
assert_equal \
    "SV family-status main parser generation parser rejections total" \
    "$sv_family_status_systemverilog_generation_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_generation_parser_rejections_total")"
assert_equal \
    "SV family-status main parser replay-shadow parser rejections total" \
    "$sv_family_status_systemverilog_replay_shadow_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_replay_shadow_parser_rejections_total")"
assert_equal \
    "SV family-status main parser focused replay target count" \
    "$sv_family_status_systemverilog_focused_replay_target_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_focused_replay_target_count")"
assert_equal \
    "SV family-status main parser focused replay covered reachable rules" \
    "$sv_family_status_systemverilog_focused_replay_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_focused_replay_covered_reachable_rules")"
assert_equal \
    "SV family-status main parser focused replay covered reachable branches" \
    "$sv_family_status_systemverilog_focused_replay_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_focused_replay_covered_reachable_branches")"
assert_equal \
    "SV family-status main parser replay-gap primary rule" \
    "$sv_family_status_systemverilog_replay_gap_target_primary_rule" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_replay_gap_target_primary_rule")"
assert_equal \
    "SV family-status main parser semantic-scope case count" \
    "$sv_family_status_systemverilog_semantic_scope_case_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_case_count")"
assert_equal \
    "SV family-status main parser semantic-scope failed count" \
    "$sv_family_status_systemverilog_semantic_scope_failed_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_failed_count")"
assert_equal \
    "SV family-status main parser semantic-scope contract criterion" \
    "$sv_family_status_systemverilog_semantic_scope_contract_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_contract_green")"
assert_equal \
    "SV family-status main parser syntax-closure state dir" \
    "$sv_family_status_systemverilog_syntax_closure_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_state_dir")"
assert_equal \
    "SV family-status main parser syntax-closure summary txt" \
    "$sv_family_status_systemverilog_syntax_closure_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_summary_txt")"
assert_equal \
    "SV family-status main parser syntax-closure summary path" \
    "$sv_family_status_systemverilog_syntax_closure_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_syntax_closure_summary_json")"
assert_equal \
    "SV family-status main parser aggregate state dir" \
    "$sv_family_status_systemverilog_parser_aggregate_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_parser_aggregate_state_dir")"
assert_equal \
    "SV family-status main parser aggregate-summary path" \
    "$sv_family_status_systemverilog_parser_aggregate_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_parser_aggregate_summary_txt")"
assert_equal \
    "SOTA JSON main parser aggregate state dir" \
    "$sv_family_status_systemverilog_parser_aggregate_state_dir" \
    "$sota_exit_sv_systemverilog_parser_aggregate_state_dir"
assert_equal \
    "SOTA JSON main parser aggregate summary txt" \
    "$sv_family_status_systemverilog_parser_aggregate_summary_txt" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_txt"
assert_equal \
    "SOTA JSON main parser aggregate summary json" \
    "$sv_family_status_systemverilog_parser_aggregate_summary_json" \
    "$sota_exit_sv_systemverilog_parser_aggregate_summary_json"
assert_equal \
    "SV family-status main parser semantic-scope state dir" \
    "$sv_family_status_systemverilog_semantic_scope_contract_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_contract_state_dir")"
assert_equal \
    "SV family-status main parser semantic-scope summary txt" \
    "$sv_family_status_systemverilog_semantic_scope_contract_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_contract_summary_txt")"
assert_equal \
    "SV family-status main parser semantic-scope summary path" \
    "$sv_family_status_systemverilog_semantic_scope_contract_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_semantic_scope_contract_summary_json")"
assert_equal \
    "SV family-status preprocessor label" \
    "$sv_family_status_systemverilog_preprocessor" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor")"
assert_equal \
    "SV family-status preprocessor tracker status" \
    "$sv_family_status_systemverilog_preprocessor_tracker_status" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_tracker_status")"
assert_equal \
    "SV family-status preprocessor tracker alignment" \
    "$sv_family_status_systemverilog_preprocessor_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_tracker_alignment_ok")"
assert_equal \
    "SV family-status preprocessor unmet closure criteria count" \
    "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count")"
assert_equal \
    "SV family-status preprocessor unmet closure criteria json" \
    "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json")"
assert_equal \
    "SV family-status preprocessor unmet closure criteria details json" \
    "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json")"
assert_equal \
    "SV family-status preprocessor closure criteria total count" \
    "$sv_family_status_systemverilog_preprocessor_closure_criteria_total_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_closure_criteria_total_count")"
assert_equal \
    "SV family-status preprocessor closure criteria satisfied count" \
    "$sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count")"
assert_equal \
    "SV family-status preprocessor closure criteria unsatisfied count" \
    "$sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count")"
assert_equal \
    "SV family-status preprocessor primary unmet closure criterion" \
    "$sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion")"
assert_equal \
    "SV family-status preprocessor syntax-closure status" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_status" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_status")"
assert_equal \
    "SV family-status preprocessor syntax-closure failure count" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_failure_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_failure_count")"
assert_equal \
    "SV family-status preprocessor syntax defined rule count" \
    "$sv_family_status_systemverilog_preprocessor_syntax_defined_rule_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_defined_rule_count")"
assert_equal \
    "SV family-status preprocessor syntax unresolved rule reference count" \
    "$sv_family_status_systemverilog_preprocessor_syntax_unresolved_rule_reference_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_unresolved_rule_reference_count")"
assert_equal \
    "SV family-status preprocessor syntax unreachable rules" \
    "$sv_family_status_systemverilog_preprocessor_syntax_unreachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_unreachable_rules")"
assert_equal \
    "SV family-status preprocessor syntax unreachable branches" \
    "$sv_family_status_systemverilog_preprocessor_syntax_unreachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_unreachable_branches")"
assert_equal \
    "SV family-status preprocessor syntax target debt count" \
    "$sv_family_status_systemverilog_preprocessor_syntax_target_debt_count" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_target_debt_count")"
assert_equal \
    "SV family-status preprocessor syntax-closure gate green" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_gate_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_gate_green")"
assert_equal \
    "SV family-status preprocessor aggregate-contract green" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_contract_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_aggregate_contract_green")"
assert_equal \
    "SV family-status preprocessor reachability-closure green" \
    "$sv_family_status_systemverilog_preprocessor_reachability_closure_green" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_closure_green")"
assert_equal \
    "SV family-status preprocessor parser-rejections zero" \
    "$sv_family_status_systemverilog_preprocessor_parser_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_parser_rejections_zero")"
assert_equal \
    "SV family-status preprocessor parseability-rejections zero" \
    "$sv_family_status_systemverilog_preprocessor_parseability_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_parseability_rejections_zero")"
assert_equal \
    "SV family-status preprocessor reachability stage3 targets zero" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_targets_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_targets_zero")"
assert_equal \
    "SV family-status preprocessor reachability stage4 targets zero" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_targets_zero" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_targets_zero")"
assert_equal \
    "SV family-status preprocessor reachability stage3 rules full" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_rules_full" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_rules_full")"
assert_equal \
    "SV family-status preprocessor reachability stage4 rules full" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_rules_full" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_rules_full")"
assert_equal \
    "SV family-status preprocessor reachability stage3 branches full" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_branches_full" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_branches_full")"
assert_equal \
    "SV family-status preprocessor reachability stage4 branches full" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_branches_full" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_branches_full")"
assert_equal \
    "SV family-status preprocessor parseability parser rejections total" \
    "$sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total")"
assert_equal \
    "SV family-status preprocessor parseability rejected total" \
    "$sv_family_status_systemverilog_preprocessor_parseability_rejected_total" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_parseability_rejected_total")"
assert_equal \
    "SV family-status preprocessor final targets" \
    "$sv_family_status_systemverilog_preprocessor_final_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_final_targets")"
assert_equal \
    "SV family-status preprocessor covered reachable rules" \
    "$sv_family_status_systemverilog_preprocessor_covered_reachable_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_covered_reachable_rules")"
assert_equal \
    "SV family-status preprocessor covered reachable branches" \
    "$sv_family_status_systemverilog_preprocessor_covered_reachable_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_covered_reachable_branches")"
assert_equal \
    "SV family-status preprocessor counterexample primary stage" \
    "$sv_family_status_systemverilog_preprocessor_counterexample_primary_stage" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_counterexample_primary_stage")"
assert_equal \
    "SV family-status preprocessor reachability stage3 targets" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_targets")"
assert_equal \
    "SV family-status preprocessor reachability stage4 targets" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_targets" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_targets")"
assert_equal \
    "SV family-status preprocessor reachability stage3 rules" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_rules")"
assert_equal \
    "SV family-status preprocessor reachability stage4 rules" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_rules" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_rules")"
assert_equal \
    "SV family-status preprocessor reachability stage3 branches" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage3_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage3_branches")"
assert_equal \
    "SV family-status preprocessor reachability stage4 branches" \
    "$sv_family_status_systemverilog_preprocessor_reachability_stage4_branches" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_stage4_branches")"
assert_equal \
    "SV family-status preprocessor syntax-closure state dir" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir")"
assert_equal \
    "SV family-status preprocessor syntax-closure summary txt" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt")"
assert_equal \
    "SV family-status preprocessor syntax-closure summary path" \
    "$sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json")"
assert_equal \
    "SV family-status preprocessor aggregate state dir" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_aggregate_state_dir")"
assert_equal \
    "SV family-status preprocessor aggregate-summary path" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_aggregate_summary_txt")"
assert_equal \
    "SOTA JSON preprocessor aggregate state dir" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_state_dir" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_state_dir"
assert_equal \
    "SOTA JSON preprocessor aggregate summary txt" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_summary_txt" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_txt"
assert_equal \
    "SOTA JSON preprocessor aggregate summary json" \
    "$sv_family_status_systemverilog_preprocessor_aggregate_summary_json" \
    "$sota_exit_sv_systemverilog_preprocessor_aggregate_summary_json"
assert_equal \
    "SV family-status preprocessor reachability state dir" \
    "$sv_family_status_systemverilog_preprocessor_reachability_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_state_dir")"
assert_equal \
    "SV family-status preprocessor reachability-summary path" \
    "$sv_family_status_systemverilog_preprocessor_reachability_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_systemverilog_preprocessor_reachability_summary_txt")"
assert_equal \
    "SOTA JSON main contract parser aggregate state dir" \
    "$sv_family_status_contract_systemverilog_parser_aggregate_state_dir" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_state_dir"
assert_equal \
    "SOTA JSON main contract parser aggregate summary txt" \
    "$sv_family_status_contract_systemverilog_parser_aggregate_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_txt"
assert_equal \
    "SOTA JSON main contract parser aggregate summary json" \
    "$sv_family_status_contract_systemverilog_parser_aggregate_summary_json" \
    "$sota_exit_sv_contract_systemverilog_parser_aggregate_summary_json"
assert_equal \
    "SV family-status contract semantic-scope state dir" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir")"
assert_equal \
    "SV family-status contract semantic-scope summary txt" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt")"
assert_equal \
    "SV family-status contract semantic-scope summary json" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json")"
assert_equal \
    "SOTA JSON main contract semantic-scope state dir" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_state_dir"
assert_equal \
    "SOTA JSON main contract semantic-scope summary txt" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_txt"
assert_equal \
    "SOTA JSON main contract semantic-scope summary json" \
    "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json" \
    "$sota_exit_sv_contract_systemverilog_semantic_scope_contract_summary_json"
assert_equal \
    "SOTA JSON preprocessor contract aggregate state dir" \
    "$sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_state_dir"
assert_equal \
    "SOTA JSON preprocessor contract aggregate summary txt" \
    "$sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_txt"
assert_equal \
    "SOTA JSON preprocessor contract aggregate summary json" \
    "$sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json" \
    "$sota_exit_sv_contract_systemverilog_preprocessor_aggregate_summary_json"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
    echo "existing_sota_exit_state_dir: ${EXISTING_SOTA_EXIT_STATE_DIR:-<unset>}"
    echo "sota_exit_state_dir: $sota_state_dir"
    echo "sota_exit_summary_txt: $sota_summary_txt"
    echo "sota_exit_summary_json: $sota_summary_json"
    echo "sota_exit_gate: $sota_exit_gate_name"
    echo "sota_exit_gate_version: $sota_exit_gate_version"
    echo "sota_exit_generated_at_utc: $sota_exit_generated_at_utc"
    echo "sota_exit_status: $sota_exit_status"
    echo "sota_exit_required_failures: $sota_exit_required_failures"
    echo "sota_exit_informational_failures: $sota_exit_informational_failures"
    echo "sota_exit_all_failures: $sota_exit_all_failures"
    echo "sv_stimuli_quality_aggregate_contract_summary_txt: $sv_parser_aggregate_summary_txt"
    echo "sv_preprocessor_quality_aggregate_contract_summary_txt: $sv_preprocessor_aggregate_summary_txt"
    echo "sv_failure_context_contract_summary_txt: $sv_failure_summary_txt"
    echo "sv_failure_context_contract_summary_json: $sv_failure_summary_json"
    echo "sv_failure_context_contract_gate: $sv_failure_gate_name"
    echo "sv_failure_context_contract_gate_version: $sv_failure_gate_version"
    echo "sv_failure_context_contract_generated_at_utc: $sv_failure_generated_at_utc"
    echo "sv_roundtrip_contract_summary_txt: $sv_roundtrip_summary_txt"
    echo "sv_roundtrip_contract_summary_json: $sv_roundtrip_summary_json"
    echo "sv_roundtrip_contract_gate: $sv_roundtrip_gate_name"
    echo "sv_roundtrip_contract_gate_version: $sv_roundtrip_gate_version"
    echo "sv_roundtrip_contract_generated_at_utc: $sv_roundtrip_generated_at_utc"
    echo "sv_preprocessor_reachability_closure_summary_txt: $sv_preprocessor_reachability_summary_txt"
    echo "sv_parser_family_status_summary_txt: $sv_parser_family_status_summary_txt"
    echo "sv_parser_family_status_summary_json: $sv_parser_family_status_summary_json"
    echo "sv_parser_family_status_contract_summary_txt: $sv_parser_family_status_contract_summary_txt"
    echo "sv_parser_family_status_contract_summary_json: $sv_parser_family_status_contract_summary_json"
    echo "sv_parser_family_status_gate: $sv_parser_family_status_gate_name"
    echo "sv_parser_family_status_gate_version: $sv_parser_family_status_gate_version"
    echo "sv_parser_family_status_generated_at_utc: $sv_parser_family_status_generated_at_utc"
    echo "sv_parser_family_status_live_tracker_file: $sv_parser_family_status_live_tracker_file"
    echo "sv_parser_family_status_status_rule_done: $sv_parser_family_status_status_rule_done"
    echo "sv_family_status_contract_gate: $sv_family_status_contract_gate"
    echo "sv_family_status_contract_gate_version: $sv_family_status_contract_gate_version"
    echo "sv_family_status_contract_generated_at_utc: $sv_family_status_contract_generated_at_utc"
    echo "sv_family_status_contract_family_status_state_dir: $sv_family_status_contract_family_status_state_dir"
    echo "sv_family_status_contract_family_status_summary_json: $sv_family_status_contract_family_status_summary_json"
    echo "sv_family_status_contract_family_status_summary_txt: $sv_family_status_contract_family_status_summary_txt"
    echo "sv_family_status_contract_systemverilog_parser_aggregate_state_dir: $sv_family_status_contract_systemverilog_parser_aggregate_state_dir"
    echo "sv_family_status_contract_systemverilog_parser_aggregate_summary_txt: $sv_family_status_contract_systemverilog_parser_aggregate_summary_txt"
    echo "sv_family_status_contract_systemverilog_parser_aggregate_summary_json: $sv_family_status_contract_systemverilog_parser_aggregate_summary_json"
    echo "sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir: $sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir"
    echo "sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt"
    echo "sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json"
    echo "sv_parser_family_status_contract_family_count: $sv_parser_family_status_contract_family_count"
    echo "sv_parser_family_status_contract_systemverilog_tracker_alignment_ok: $sv_parser_family_status_contract_systemverilog_tracker_alignment_ok"
    echo "sv_parser_family_status_contract_systemverilog_false_criteria_count: $sv_parser_family_status_contract_systemverilog_false_criteria_count"
    echo "sv_parser_family_status_contract_systemverilog_unmet_details_count: $sv_parser_family_status_contract_systemverilog_unmet_details_count"
    echo "sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion: $sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion"
    echo "sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json: $sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json"
    echo "sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json: $sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json"
    echo "sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir: $sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir"
    echo "sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt: $sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt"
    echo "sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json: $sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok: $sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count: $sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion: $sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json"
    echo "sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json"
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
    echo "sv_replay_gap_source_gap_json: $sv_replay_gap_source_gap_json"
    echo "sv_base_contract_file: $sv_base_contract_file"
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
    echo "sv_focused_initial_target_count: $sv_focused_initial_target_count"
    echo "sv_focused_replay_target_count: $sv_focused_replay_target_count"
    echo "sv_focused_initial_covered_reachable_rules: $sv_focused_initial_covered_reachable_rules"
    echo "sv_focused_replay_covered_reachable_rules: $sv_focused_replay_covered_reachable_rules"
    echo "sv_focused_initial_covered_reachable_branches: $sv_focused_initial_covered_reachable_branches"
    echo "sv_focused_replay_covered_reachable_branches: $sv_focused_replay_covered_reachable_branches"
    echo "sv_failure_context_generation_excerpts: $sv_failure_generation_excerpts"
    echo "sv_failure_context_shadow_excerpts: $sv_failure_shadow_excerpts"
    echo "sv_roundtrip_initial_targets: $sv_roundtrip_initial_targets"
    echo "sv_roundtrip_replay_targets: $sv_roundtrip_replay_targets"
    echo "sv_roundtrip_initial_covered_reachable_rules: $sv_roundtrip_initial_rules"
    echo "sv_roundtrip_replay_covered_reachable_rules: $sv_roundtrip_replay_rules"
    echo "sv_roundtrip_initial_covered_reachable_branches: $sv_roundtrip_initial_branches"
    echo "sv_roundtrip_replay_covered_reachable_branches: $sv_roundtrip_replay_branches"
    echo "sv_preprocessor_failure_context_excerpts: $svpp_failure_excerpts"
    echo "sv_preprocessor_parseability_report_json: $svpp_parseability_report_json"
    echo "sv_preprocessor_gap_stage3_json: $svpp_gap_stage3_json"
    echo "sv_preprocessor_parseability_attempts_total: $svpp_parseability_attempts_total"
    echo "sv_preprocessor_parseability_accepted_total: $svpp_parseability_accepted_total"
    echo "sv_preprocessor_parseability_rejected_total: $svpp_parseability_rejected_total"
    echo "sv_preprocessor_parseability_parser_rejections_total: $svpp_parseability_parser_rejections_total"
    echo "sv_preprocessor_parseability_counterexamples_captured_total: $svpp_parseability_counterexamples_captured_total"
    echo "sv_preprocessor_stage0_target_count: $svpp_stage0_target_count"
    echo "sv_preprocessor_stage1_target_count: $svpp_stage1_target_count"
    echo "sv_preprocessor_final_targets: $svpp_final_target_count"
    echo "sv_preprocessor_stage4_target_count: $svpp_stage4_target_count"
    echo "sv_preprocessor_stage0_covered_reachable_rules: $svpp_stage0_covered_reachable_rules"
    echo "sv_preprocessor_stage1_covered_reachable_rules: $svpp_stage1_covered_reachable_rules"
    echo "sv_preprocessor_covered_reachable_rules: $svpp_covered_reachable_rules"
    echo "sv_preprocessor_stage4_covered_reachable_rules: $svpp_stage4_covered_reachable_rules"
    echo "sv_preprocessor_stage0_covered_reachable_branches: $svpp_stage0_covered_reachable_branches"
    echo "sv_preprocessor_stage1_covered_reachable_branches: $svpp_stage1_covered_reachable_branches"
    echo "sv_preprocessor_covered_reachable_branches: $svpp_covered_reachable_branches"
    echo "sv_preprocessor_stage4_covered_reachable_branches: $svpp_stage4_covered_reachable_branches"
    echo "sv_preprocessor_fuzz_replay_accepted_cases: $svpp_fuzz_replay_accepted_cases"
    echo "sv_preprocessor_fuzz_replay_rejected_cases: $svpp_fuzz_replay_rejected_cases"
    echo "sv_preprocessor_fuzz_replay_parseability_counterexamples: $svpp_fuzz_replay_parseability_counterexamples"
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
    echo "sv_family_status_systemverilog: $sv_family_status_systemverilog"
    echo "sv_family_status_systemverilog_tracker_status: $sv_family_status_systemverilog_tracker_status"
    echo "sv_family_status_systemverilog_tracker_alignment_ok: $sv_family_status_systemverilog_tracker_alignment_ok"
    echo "sv_family_status_systemverilog_unmet_closure_criteria_count: $sv_family_status_systemverilog_unmet_closure_criteria_count"
    echo "sv_family_status_systemverilog_unmet_closure_criteria_json: $sv_family_status_systemverilog_unmet_closure_criteria_json"
    echo "sv_family_status_systemverilog_unmet_closure_criteria_details_json: $sv_family_status_systemverilog_unmet_closure_criteria_details_json"
    echo "sv_family_status_systemverilog_primary_unmet_closure_criterion: $sv_family_status_systemverilog_primary_unmet_closure_criterion"
    echo "sv_family_status_systemverilog_closure_criteria_total_count: $sv_family_status_systemverilog_closure_criteria_total_count"
    echo "sv_family_status_systemverilog_closure_criteria_satisfied_count: $sv_family_status_systemverilog_closure_criteria_satisfied_count"
    echo "sv_family_status_systemverilog_closure_criteria_unsatisfied_count: $sv_family_status_systemverilog_closure_criteria_unsatisfied_count"
    echo "sv_family_status_systemverilog_syntax_closure_status: $sv_family_status_systemverilog_syntax_closure_status"
    echo "sv_family_status_systemverilog_syntax_closure_failure_count: $sv_family_status_systemverilog_syntax_closure_failure_count"
    echo "sv_family_status_systemverilog_syntax_defined_rule_count: $sv_family_status_systemverilog_syntax_defined_rule_count"
    echo "sv_family_status_systemverilog_syntax_unresolved_rule_reference_count: $sv_family_status_systemverilog_syntax_unresolved_rule_reference_count"
    echo "sv_family_status_systemverilog_syntax_unreachable_rules: $sv_family_status_systemverilog_syntax_unreachable_rules"
    echo "sv_family_status_systemverilog_syntax_unreachable_branches: $sv_family_status_systemverilog_syntax_unreachable_branches"
    echo "sv_family_status_systemverilog_syntax_target_debt_count: $sv_family_status_systemverilog_syntax_target_debt_count"
    echo "sv_family_status_systemverilog_syntax_closure_gate_green: $sv_family_status_systemverilog_syntax_closure_gate_green"
    echo "sv_family_status_systemverilog_parser_aggregate_contract_green: $sv_family_status_systemverilog_parser_aggregate_contract_green"
    echo "sv_family_status_systemverilog_generation_parser_rejections_zero: $sv_family_status_systemverilog_generation_parser_rejections_zero"
    echo "sv_family_status_systemverilog_replay_shadow_parser_rejections_zero: $sv_family_status_systemverilog_replay_shadow_parser_rejections_zero"
    echo "sv_family_status_systemverilog_focused_replay_target_debt_zero: $sv_family_status_systemverilog_focused_replay_target_debt_zero"
    echo "sv_family_status_systemverilog_generation_parser_rejections_total: $sv_family_status_systemverilog_generation_parser_rejections_total"
    echo "sv_family_status_systemverilog_replay_shadow_parser_rejections_total: $sv_family_status_systemverilog_replay_shadow_parser_rejections_total"
    echo "sv_family_status_systemverilog_focused_replay_target_count: $sv_family_status_systemverilog_focused_replay_target_count"
    echo "sv_family_status_systemverilog_focused_replay_covered_reachable_rules: $sv_family_status_systemverilog_focused_replay_covered_reachable_rules"
    echo "sv_family_status_systemverilog_focused_replay_covered_reachable_branches: $sv_family_status_systemverilog_focused_replay_covered_reachable_branches"
    echo "sv_family_status_systemverilog_replay_gap_target_primary_rule: $sv_family_status_systemverilog_replay_gap_target_primary_rule"
    echo "sv_family_status_systemverilog_semantic_scope_case_count: $sv_family_status_systemverilog_semantic_scope_case_count"
    echo "sv_family_status_systemverilog_semantic_scope_failed_count: $sv_family_status_systemverilog_semantic_scope_failed_count"
    echo "sv_family_status_systemverilog_semantic_scope_contract_green: $sv_family_status_systemverilog_semantic_scope_contract_green"
    echo "sv_family_status_systemverilog_syntax_closure_state_dir: $sv_family_status_systemverilog_syntax_closure_state_dir"
    echo "sv_family_status_systemverilog_syntax_closure_summary_txt: $sv_family_status_systemverilog_syntax_closure_summary_txt"
    echo "sv_family_status_systemverilog_syntax_closure_summary_json: $sv_family_status_systemverilog_syntax_closure_summary_json"
    echo "sv_family_status_systemverilog_parser_aggregate_state_dir: $sv_family_status_systemverilog_parser_aggregate_state_dir"
    echo "sv_family_status_systemverilog_parser_aggregate_summary_txt: $sv_family_status_systemverilog_parser_aggregate_summary_txt"
    echo "sv_family_status_systemverilog_parser_aggregate_summary_json: $sv_family_status_systemverilog_parser_aggregate_summary_json"
    echo "sv_family_status_systemverilog_semantic_scope_contract_state_dir: $sv_family_status_systemverilog_semantic_scope_contract_state_dir"
    echo "sv_family_status_systemverilog_semantic_scope_contract_summary_txt: $sv_family_status_systemverilog_semantic_scope_contract_summary_txt"
    echo "sv_family_status_systemverilog_semantic_scope_contract_summary_json: $sv_family_status_systemverilog_semantic_scope_contract_summary_json"
    echo "sv_family_status_systemverilog_preprocessor: $sv_family_status_systemverilog_preprocessor"
    echo "sv_family_status_systemverilog_preprocessor_tracker_status: $sv_family_status_systemverilog_preprocessor_tracker_status"
    echo "sv_family_status_systemverilog_preprocessor_tracker_alignment_ok: $sv_family_status_systemverilog_preprocessor_tracker_alignment_ok"
    echo "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count"
    echo "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json"
    echo "sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json"
    echo "sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion: $sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion"
    echo "sv_family_status_systemverilog_preprocessor_closure_criteria_total_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_total_count"
    echo "sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count"
    echo "sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_status: $sv_family_status_systemverilog_preprocessor_syntax_closure_status"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_failure_count: $sv_family_status_systemverilog_preprocessor_syntax_closure_failure_count"
    echo "sv_family_status_systemverilog_preprocessor_syntax_defined_rule_count: $sv_family_status_systemverilog_preprocessor_syntax_defined_rule_count"
    echo "sv_family_status_systemverilog_preprocessor_syntax_unresolved_rule_reference_count: $sv_family_status_systemverilog_preprocessor_syntax_unresolved_rule_reference_count"
    echo "sv_family_status_systemverilog_preprocessor_syntax_unreachable_rules: $sv_family_status_systemverilog_preprocessor_syntax_unreachable_rules"
    echo "sv_family_status_systemverilog_preprocessor_syntax_unreachable_branches: $sv_family_status_systemverilog_preprocessor_syntax_unreachable_branches"
    echo "sv_family_status_systemverilog_preprocessor_syntax_target_debt_count: $sv_family_status_systemverilog_preprocessor_syntax_target_debt_count"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_gate_green: $sv_family_status_systemverilog_preprocessor_syntax_closure_gate_green"
    echo "sv_family_status_systemverilog_preprocessor_aggregate_contract_green: $sv_family_status_systemverilog_preprocessor_aggregate_contract_green"
    echo "sv_family_status_systemverilog_preprocessor_reachability_closure_green: $sv_family_status_systemverilog_preprocessor_reachability_closure_green"
    echo "sv_family_status_systemverilog_preprocessor_parser_rejections_zero: $sv_family_status_systemverilog_preprocessor_parser_rejections_zero"
    echo "sv_family_status_systemverilog_preprocessor_parseability_rejections_zero: $sv_family_status_systemverilog_preprocessor_parseability_rejections_zero"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_targets_zero: $sv_family_status_systemverilog_preprocessor_reachability_stage3_targets_zero"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_targets_zero: $sv_family_status_systemverilog_preprocessor_reachability_stage4_targets_zero"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_rules_full: $sv_family_status_systemverilog_preprocessor_reachability_stage3_rules_full"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_rules_full: $sv_family_status_systemverilog_preprocessor_reachability_stage4_rules_full"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_branches_full: $sv_family_status_systemverilog_preprocessor_reachability_stage3_branches_full"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_branches_full: $sv_family_status_systemverilog_preprocessor_reachability_stage4_branches_full"
    echo "sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total: $sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total"
    echo "sv_family_status_systemverilog_preprocessor_parseability_rejected_total: $sv_family_status_systemverilog_preprocessor_parseability_rejected_total"
    echo "sv_family_status_systemverilog_preprocessor_final_targets: $sv_family_status_systemverilog_preprocessor_final_targets"
    echo "sv_family_status_systemverilog_preprocessor_covered_reachable_rules: $sv_family_status_systemverilog_preprocessor_covered_reachable_rules"
    echo "sv_family_status_systemverilog_preprocessor_covered_reachable_branches: $sv_family_status_systemverilog_preprocessor_covered_reachable_branches"
    echo "sv_family_status_systemverilog_preprocessor_counterexample_primary_stage: $sv_family_status_systemverilog_preprocessor_counterexample_primary_stage"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_targets: $sv_family_status_systemverilog_preprocessor_reachability_stage3_targets"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_targets: $sv_family_status_systemverilog_preprocessor_reachability_stage4_targets"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_rules: $sv_family_status_systemverilog_preprocessor_reachability_stage3_rules"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_rules: $sv_family_status_systemverilog_preprocessor_reachability_stage4_rules"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage3_branches: $sv_family_status_systemverilog_preprocessor_reachability_stage3_branches"
    echo "sv_family_status_systemverilog_preprocessor_reachability_stage4_branches: $sv_family_status_systemverilog_preprocessor_reachability_stage4_branches"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir: $sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt: $sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt"
    echo "sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json: $sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json"
    echo "sv_family_status_systemverilog_preprocessor_aggregate_state_dir: $sv_family_status_systemverilog_preprocessor_aggregate_state_dir"
    echo "sv_family_status_systemverilog_preprocessor_aggregate_summary_txt: $sv_family_status_systemverilog_preprocessor_aggregate_summary_txt"
    echo "sv_family_status_systemverilog_preprocessor_aggregate_summary_json: $sv_family_status_systemverilog_preprocessor_aggregate_summary_json"
    echo "sv_family_status_systemverilog_preprocessor_reachability_state_dir: $sv_family_status_systemverilog_preprocessor_reachability_state_dir"
    echo "sv_family_status_systemverilog_preprocessor_reachability_summary_txt: $sv_family_status_systemverilog_preprocessor_reachability_summary_txt"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_combined_telemetry_contract_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg sv_contract_file "$SV_CONTRACT_FILE" \
    --arg sota_policy_env_file "$SOTA_POLICY_ENV_FILE" \
    --arg existing_sota_exit_state_dir "${EXISTING_SOTA_EXIT_STATE_DIR:-}" \
    --arg sota_exit_state_dir "$sota_state_dir" \
    --arg sota_exit_summary_txt "$sota_summary_txt" \
    --arg sota_exit_summary_json "$sota_summary_json" \
    --arg sota_exit_gate "$sota_exit_gate_name" \
    --argjson sota_exit_gate_version "$sota_exit_gate_version" \
    --arg sota_exit_generated_at_utc "$sota_exit_generated_at_utc" \
    --arg sota_exit_status "$sota_exit_status" \
    --argjson sota_exit_required_failures "$sota_exit_required_failures" \
    --argjson sota_exit_informational_failures "$sota_exit_informational_failures" \
    --argjson sota_exit_all_failures "$sota_exit_all_failures" \
    --arg sv_stimuli_quality_aggregate_contract_summary_txt "$sv_parser_aggregate_summary_txt" \
    --arg sv_preprocessor_quality_aggregate_contract_summary_txt "$sv_preprocessor_aggregate_summary_txt" \
    --arg sv_failure_context_contract_summary_txt "$sv_failure_summary_txt" \
    --arg sv_failure_context_contract_summary_json "$sv_failure_summary_json" \
    --arg sv_failure_context_contract_gate "$sv_failure_gate_name" \
    --argjson sv_failure_context_contract_gate_version "$sv_failure_gate_version" \
    --arg sv_failure_context_contract_generated_at_utc "$sv_failure_generated_at_utc" \
    --arg sv_roundtrip_contract_summary_txt "$sv_roundtrip_summary_txt" \
    --arg sv_roundtrip_contract_summary_json "$sv_roundtrip_summary_json" \
    --arg sv_roundtrip_contract_gate "$sv_roundtrip_gate_name" \
    --argjson sv_roundtrip_contract_gate_version "$sv_roundtrip_gate_version" \
    --arg sv_roundtrip_contract_generated_at_utc "$sv_roundtrip_generated_at_utc" \
    --arg sv_preprocessor_reachability_closure_summary_txt "$sv_preprocessor_reachability_summary_txt" \
    --arg sv_parser_family_status_summary_txt "$sv_parser_family_status_summary_txt" \
    --arg sv_parser_family_status_summary_json "$sv_parser_family_status_summary_json" \
    --arg sv_parser_family_status_contract_summary_txt "$sv_parser_family_status_contract_summary_txt" \
    --arg sv_parser_family_status_contract_summary_json "$sv_parser_family_status_contract_summary_json" \
    --argjson sv_failure_context_summary_available "$sv_failure_summary_available" \
    --argjson sv_roundtrip_summary_available "$sv_roundtrip_summary_available" \
    --argjson sv_failure_generation_excerpts "$sv_failure_generation_excerpts" \
    --argjson sv_failure_shadow_excerpts "$sv_failure_shadow_excerpts" \
    --argjson svpp_failure_excerpts "$svpp_failure_excerpts" \
    --argjson sv_roundtrip_initial_targets "$sv_roundtrip_initial_targets" \
    --argjson sv_roundtrip_replay_targets "$sv_roundtrip_replay_targets" \
    --argjson sv_roundtrip_initial_rules "$sv_roundtrip_initial_rules" \
    --argjson sv_roundtrip_replay_rules "$sv_roundtrip_replay_rules" \
    --argjson sv_roundtrip_initial_branches "$sv_roundtrip_initial_branches" \
    --argjson sv_roundtrip_replay_branches "$sv_roundtrip_replay_branches" \
    --argjson svpp_stage0_targets "$svpp_stage0_targets" \
    --argjson svpp_stage1_targets "$svpp_stage1_targets" \
    --argjson svpp_final_targets "$svpp_final_targets" \
    --argjson svpp_stage4_targets "$svpp_stage4_targets" \
    --arg svpp_stage0_rules "$svpp_stage0_rules" \
    --arg svpp_stage1_rules "$svpp_stage1_rules" \
    --arg svpp_stage4_rules "$svpp_stage4_rules" \
    --arg svpp_stage0_branches "$svpp_stage0_branches" \
    --arg svpp_stage1_branches "$svpp_stage1_branches" \
    --arg svpp_stage4_branches "$svpp_stage4_branches" \
    --arg sv_parser_family_status_gate "$sv_parser_family_status_gate_name" \
    --argjson sv_parser_family_status_gate_version "$sv_parser_family_status_gate_version" \
    --arg sv_parser_family_status_generated_at_utc "$sv_parser_family_status_generated_at_utc" \
    --arg sv_parser_family_status_live_tracker_file "$sv_parser_family_status_live_tracker_file" \
    --arg sv_parser_family_status_status_rule_done "$sv_parser_family_status_status_rule_done" \
    --arg sv_family_status_systemverilog "$sv_family_status_systemverilog" \
    --arg sv_family_status_systemverilog_tracker_status "$sv_family_status_systemverilog_tracker_status" \
    --argjson sv_family_status_systemverilog_tracker_alignment_ok "$sv_family_status_systemverilog_tracker_alignment_ok" \
    --argjson sv_family_status_systemverilog_unmet_closure_criteria_count "$sv_family_status_systemverilog_unmet_closure_criteria_count" \
    --arg sv_family_status_systemverilog_primary_unmet_closure_criterion "$sv_family_status_systemverilog_primary_unmet_closure_criterion" \
    --argjson sv_family_status_systemverilog_unmet_closure_criteria_json "$sv_family_status_systemverilog_unmet_closure_criteria_json" \
    --argjson sv_family_status_systemverilog_unmet_closure_criteria_details_json "$sv_family_status_systemverilog_unmet_closure_criteria_details_json" \
    --argjson sv_family_status_systemverilog_closure_criteria_total_count "$sv_family_status_systemverilog_closure_criteria_total_count" \
    --argjson sv_family_status_systemverilog_closure_criteria_satisfied_count "$sv_family_status_systemverilog_closure_criteria_satisfied_count" \
    --argjson sv_family_status_systemverilog_closure_criteria_unsatisfied_count "$sv_family_status_systemverilog_closure_criteria_unsatisfied_count" \
    --argjson sv_family_status_systemverilog_syntax_target_debt_count "$sv_family_status_systemverilog_syntax_target_debt_count" \
    --argjson sv_family_status_systemverilog_generation_parser_rejections_total "$sv_family_status_systemverilog_generation_parser_rejections_total" \
    --argjson sv_family_status_systemverilog_replay_shadow_parser_rejections_total "$sv_family_status_systemverilog_replay_shadow_parser_rejections_total" \
    --argjson sv_family_status_systemverilog_focused_replay_target_count "$sv_family_status_systemverilog_focused_replay_target_count" \
    --argjson sv_family_status_systemverilog_focused_replay_covered_reachable_rules "$sv_family_status_systemverilog_focused_replay_covered_reachable_rules" \
    --argjson sv_family_status_systemverilog_focused_replay_covered_reachable_branches "$sv_family_status_systemverilog_focused_replay_covered_reachable_branches" \
    --argjson sv_family_status_systemverilog_semantic_scope_case_count "$sv_family_status_systemverilog_semantic_scope_case_count" \
    --argjson sv_family_status_systemverilog_semantic_scope_failed_count "$sv_family_status_systemverilog_semantic_scope_failed_count" \
    --arg sv_family_status_systemverilog_syntax_closure_state_dir "$sv_family_status_systemverilog_syntax_closure_state_dir" \
    --arg sv_family_status_systemverilog_syntax_closure_summary_txt "$sv_family_status_systemverilog_syntax_closure_summary_txt" \
    --arg sv_family_status_systemverilog_syntax_closure_summary_json "$sv_family_status_systemverilog_syntax_closure_summary_json" \
    --arg sv_family_status_systemverilog_parser_aggregate_state_dir "$sv_family_status_systemverilog_parser_aggregate_state_dir" \
    --arg sv_family_status_systemverilog_parser_aggregate_summary_txt "$sv_family_status_systemverilog_parser_aggregate_summary_txt" \
    --arg sv_family_status_systemverilog_parser_aggregate_summary_json "$sv_family_status_systemverilog_parser_aggregate_summary_json" \
    --arg sv_family_status_systemverilog_semantic_scope_contract_state_dir "$sv_family_status_systemverilog_semantic_scope_contract_state_dir" \
    --arg sv_family_status_systemverilog_semantic_scope_contract_summary_txt "$sv_family_status_systemverilog_semantic_scope_contract_summary_txt" \
    --arg sv_family_status_systemverilog_semantic_scope_contract_summary_json "$sv_family_status_systemverilog_semantic_scope_contract_summary_json" \
    --arg sv_family_status_systemverilog_preprocessor "$sv_family_status_systemverilog_preprocessor" \
    --arg sv_family_status_systemverilog_preprocessor_tracker_status "$sv_family_status_systemverilog_preprocessor_tracker_status" \
    --argjson sv_family_status_systemverilog_preprocessor_tracker_alignment_ok "$sv_family_status_systemverilog_preprocessor_tracker_alignment_ok" \
    --argjson sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count" \
    --arg sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion "$sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion" \
    --argjson sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json" \
    --argjson sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json "$sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    --argjson sv_family_status_systemverilog_preprocessor_closure_criteria_total_count "$sv_family_status_systemverilog_preprocessor_closure_criteria_total_count" \
    --argjson sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count "$sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count" \
    --argjson sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count "$sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count" \
    --argjson sv_family_status_systemverilog_preprocessor_syntax_target_debt_count "$sv_family_status_systemverilog_preprocessor_syntax_target_debt_count" \
    --argjson sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total "$sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total" \
    --argjson sv_family_status_systemverilog_preprocessor_parseability_rejected_total "$sv_family_status_systemverilog_preprocessor_parseability_rejected_total" \
    --argjson sv_family_status_systemverilog_preprocessor_final_targets "$sv_family_status_systemverilog_preprocessor_final_targets" \
    --arg sv_family_status_systemverilog_preprocessor_covered_reachable_rules "$sv_family_status_systemverilog_preprocessor_covered_reachable_rules" \
    --arg sv_family_status_systemverilog_preprocessor_covered_reachable_branches "$sv_family_status_systemverilog_preprocessor_covered_reachable_branches" \
    --argjson sv_family_status_systemverilog_preprocessor_reachability_stage3_targets "$sv_family_status_systemverilog_preprocessor_reachability_stage3_targets" \
    --argjson sv_family_status_systemverilog_preprocessor_reachability_stage4_targets "$sv_family_status_systemverilog_preprocessor_reachability_stage4_targets" \
    --arg sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir "$sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir" \
    --arg sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt "$sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt" \
    --arg sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json "$sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json" \
    --arg sv_family_status_systemverilog_preprocessor_aggregate_state_dir "$sv_family_status_systemverilog_preprocessor_aggregate_state_dir" \
    --arg sv_family_status_systemverilog_preprocessor_aggregate_summary_txt "$sv_family_status_systemverilog_preprocessor_aggregate_summary_txt" \
    --arg sv_family_status_systemverilog_preprocessor_aggregate_summary_json "$sv_family_status_systemverilog_preprocessor_aggregate_summary_json" \
    --arg sv_family_status_systemverilog_preprocessor_reachability_state_dir "$sv_family_status_systemverilog_preprocessor_reachability_state_dir" \
    --arg sv_family_status_systemverilog_preprocessor_reachability_summary_txt "$sv_family_status_systemverilog_preprocessor_reachability_summary_txt" \
    --arg sv_family_status_contract_gate "$sv_family_status_contract_gate" \
    --argjson sv_family_status_contract_gate_version "$sv_family_status_contract_gate_version" \
    --arg sv_family_status_contract_generated_at_utc "$sv_family_status_contract_generated_at_utc" \
    --arg sv_family_status_contract_family_status_state_dir "$sv_family_status_contract_family_status_state_dir" \
    --arg sv_family_status_contract_family_status_summary_json "$sv_family_status_contract_family_status_summary_json" \
    --arg sv_family_status_contract_family_status_summary_txt "$sv_family_status_contract_family_status_summary_txt" \
    --arg sv_family_status_contract_systemverilog_parser_aggregate_state_dir "$sv_family_status_contract_systemverilog_parser_aggregate_state_dir" \
    --arg sv_family_status_contract_systemverilog_parser_aggregate_summary_txt "$sv_family_status_contract_systemverilog_parser_aggregate_summary_txt" \
    --arg sv_family_status_contract_systemverilog_parser_aggregate_summary_json "$sv_family_status_contract_systemverilog_parser_aggregate_summary_json" \
    --arg sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir "$sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir" \
    --arg sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt" \
    --arg sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json "$sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json" \
    --arg sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir "$sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir" \
    --arg sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt "$sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt" \
    --arg sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json "$sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json" \
    --argjson sv_parser_family_status_contract_family_count "$sv_parser_family_status_contract_family_count" \
    --argjson sv_parser_family_status_contract_systemverilog_tracker_alignment_ok "$sv_parser_family_status_contract_systemverilog_tracker_alignment_ok" \
    --argjson sv_parser_family_status_contract_systemverilog_false_criteria_count "$sv_parser_family_status_contract_systemverilog_false_criteria_count" \
    --argjson sv_parser_family_status_contract_systemverilog_unmet_details_count "$sv_parser_family_status_contract_systemverilog_unmet_details_count" \
    --arg sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion "$sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion" \
    --argjson sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json" \
    --argjson sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json "$sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json" \
    --argjson sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok "$sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok" \
    --argjson sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count "$sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count" \
    --argjson sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count" \
    --arg sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion "$sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion" \
    --argjson sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json" \
    --argjson sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json "$sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      sota: {
        contract_file: $sv_contract_file,
        policy_env_file: $sota_policy_env_file,
        existing_state_dir: $existing_sota_exit_state_dir,
        state_dir: $sota_exit_state_dir,
        summary_txt: $sota_exit_summary_txt,
        summary_json: $sota_exit_summary_json,
        gate: $sota_exit_gate,
        gate_version: $sota_exit_gate_version,
        generated_at_utc: $sota_exit_generated_at_utc,
        status: $sota_exit_status,
        counts: {
          required_failures: $sota_exit_required_failures,
          informational_failures: $sota_exit_informational_failures,
          all_failures: $sota_exit_all_failures
        }
      },
      proof_surfaces: {
        sota_exit_summary_txt: $sota_exit_summary_txt,
        sota_exit_summary_json: $sota_exit_summary_json,
        sv_stimuli_quality_aggregate_contract_summary_txt: $sv_stimuli_quality_aggregate_contract_summary_txt,
        sv_preprocessor_quality_aggregate_contract_summary_txt: $sv_preprocessor_quality_aggregate_contract_summary_txt,
        sv_failure_context_contract_summary_txt: $sv_failure_context_contract_summary_txt,
        sv_failure_context_contract_summary_json: $sv_failure_context_contract_summary_json,
        sv_roundtrip_contract_summary_txt: $sv_roundtrip_contract_summary_txt,
        sv_roundtrip_contract_summary_json: $sv_roundtrip_contract_summary_json,
        sv_preprocessor_reachability_closure_summary_txt: $sv_preprocessor_reachability_closure_summary_txt,
        sv_parser_family_status_summary_txt: $sv_parser_family_status_summary_txt,
        sv_parser_family_status_summary_json: $sv_parser_family_status_summary_json,
        sv_parser_family_status_contract_summary_txt: $sv_parser_family_status_contract_summary_txt,
        sv_parser_family_status_contract_summary_json: $sv_parser_family_status_contract_summary_json
      },
      optional_surfaces: {
        sv_failure_context_summary_available: $sv_failure_context_summary_available,
        sv_roundtrip_summary_available: $sv_roundtrip_summary_available
      },
      failure_context_contract: {
        gate: $sv_failure_context_contract_gate,
        version: $sv_failure_context_contract_gate_version,
        generated_at_utc: $sv_failure_context_contract_generated_at_utc,
        summary_txt: $sv_failure_context_contract_summary_txt,
        summary_json: $sv_failure_context_contract_summary_json,
        metrics: {
          systemverilog_generation_failure_context_excerpts: $sv_failure_generation_excerpts,
          systemverilog_shadow_failure_context_excerpts: $sv_failure_shadow_excerpts,
          systemverilog_preprocessor_failure_context_excerpts: $svpp_failure_excerpts
        }
      },
      roundtrip_contract: {
        gate: $sv_roundtrip_contract_gate,
        version: $sv_roundtrip_contract_gate_version,
        generated_at_utc: $sv_roundtrip_contract_generated_at_utc,
        summary_txt: $sv_roundtrip_contract_summary_txt,
        summary_json: $sv_roundtrip_contract_summary_json,
        metrics: {
          systemverilog_roundtrip_initial_targets: $sv_roundtrip_initial_targets,
          systemverilog_roundtrip_replay_targets: $sv_roundtrip_replay_targets,
          systemverilog_roundtrip_initial_covered_reachable_rules: $sv_roundtrip_initial_rules,
          systemverilog_roundtrip_replay_covered_reachable_rules: $sv_roundtrip_replay_rules,
          systemverilog_roundtrip_initial_covered_reachable_branches: $sv_roundtrip_initial_branches,
          systemverilog_roundtrip_replay_covered_reachable_branches: $sv_roundtrip_replay_branches,
          systemverilog_preprocessor_roundtrip_stage0_targets: $svpp_stage0_targets,
          systemverilog_preprocessor_roundtrip_stage1_targets: $svpp_stage1_targets,
          systemverilog_preprocessor_roundtrip_final_targets: $svpp_final_targets,
          systemverilog_preprocessor_roundtrip_stage4_targets: $svpp_stage4_targets,
          systemverilog_preprocessor_roundtrip_stage0_covered_reachable_rules: $svpp_stage0_rules,
          systemverilog_preprocessor_roundtrip_stage1_covered_reachable_rules: $svpp_stage1_rules,
          systemverilog_preprocessor_roundtrip_stage4_covered_reachable_rules: $svpp_stage4_rules,
          systemverilog_preprocessor_roundtrip_stage0_covered_reachable_branches: $svpp_stage0_branches,
          systemverilog_preprocessor_roundtrip_stage1_covered_reachable_branches: $svpp_stage1_branches,
          systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches: $svpp_stage4_branches
        }
      },
      family_status: {
        gate: $sv_parser_family_status_gate,
        version: $sv_parser_family_status_gate_version,
        generated_at_utc: $sv_parser_family_status_generated_at_utc,
        live_tracker_file: $sv_parser_family_status_live_tracker_file,
        status_rule_done: $sv_parser_family_status_status_rule_done,
        families: [
          {
            family: "systemverilog",
            computed_status: $sv_family_status_systemverilog,
            live_tracker_status: $sv_family_status_systemverilog_tracker_status,
            tracker_alignment_ok: $sv_family_status_systemverilog_tracker_alignment_ok,
            primary_unmet_closure_criterion: $sv_family_status_systemverilog_primary_unmet_closure_criterion,
            unmet_closure_criteria_count: $sv_family_status_systemverilog_unmet_closure_criteria_count,
            unmet_closure_criteria: $sv_family_status_systemverilog_unmet_closure_criteria_json,
            unmet_closure_criteria_details: $sv_family_status_systemverilog_unmet_closure_criteria_details_json,
            closure_criteria_total_count: $sv_family_status_systemverilog_closure_criteria_total_count,
            closure_criteria_satisfied_count: $sv_family_status_systemverilog_closure_criteria_satisfied_count,
            closure_criteria_unsatisfied_count: $sv_family_status_systemverilog_closure_criteria_unsatisfied_count,
            metrics: {
              syntax_target_debt_count: $sv_family_status_systemverilog_syntax_target_debt_count,
              generation_parser_rejections_total: $sv_family_status_systemverilog_generation_parser_rejections_total,
              replay_shadow_parser_rejections_total: $sv_family_status_systemverilog_replay_shadow_parser_rejections_total,
              focused_replay_target_count: $sv_family_status_systemverilog_focused_replay_target_count,
              focused_replay_covered_reachable_rules: $sv_family_status_systemverilog_focused_replay_covered_reachable_rules,
              focused_replay_covered_reachable_branches: $sv_family_status_systemverilog_focused_replay_covered_reachable_branches,
              semantic_scope_case_count: $sv_family_status_systemverilog_semantic_scope_case_count,
              semantic_scope_failed_count: $sv_family_status_systemverilog_semantic_scope_failed_count
            },
            proof_surfaces: {
              syntax_closure_state_dir: $sv_family_status_systemverilog_syntax_closure_state_dir,
              syntax_closure_summary_txt: $sv_family_status_systemverilog_syntax_closure_summary_txt,
              syntax_closure_summary_json: $sv_family_status_systemverilog_syntax_closure_summary_json,
              parser_aggregate_state_dir: $sv_family_status_systemverilog_parser_aggregate_state_dir,
              parser_aggregate_summary_txt: $sv_family_status_systemverilog_parser_aggregate_summary_txt,
              parser_aggregate_summary_json: $sv_family_status_systemverilog_parser_aggregate_summary_json,
              semantic_scope_contract_state_dir: $sv_family_status_systemverilog_semantic_scope_contract_state_dir,
              semantic_scope_contract_summary_txt: $sv_family_status_systemverilog_semantic_scope_contract_summary_txt,
              semantic_scope_contract_summary_json: $sv_family_status_systemverilog_semantic_scope_contract_summary_json
            }
          },
          {
            family: "systemverilog_preprocessor",
            computed_status: $sv_family_status_systemverilog_preprocessor,
            live_tracker_status: $sv_family_status_systemverilog_preprocessor_tracker_status,
            tracker_alignment_ok: $sv_family_status_systemverilog_preprocessor_tracker_alignment_ok,
            primary_unmet_closure_criterion: $sv_family_status_systemverilog_preprocessor_primary_unmet_closure_criterion,
            unmet_closure_criteria_count: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_count,
            unmet_closure_criteria: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_json,
            unmet_closure_criteria_details: $sv_family_status_systemverilog_preprocessor_unmet_closure_criteria_details_json,
            closure_criteria_total_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_total_count,
            closure_criteria_satisfied_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_satisfied_count,
            closure_criteria_unsatisfied_count: $sv_family_status_systemverilog_preprocessor_closure_criteria_unsatisfied_count,
            metrics: {
              syntax_target_debt_count: $sv_family_status_systemverilog_preprocessor_syntax_target_debt_count,
              parseability_parser_rejections_total: $sv_family_status_systemverilog_preprocessor_parseability_parser_rejections_total,
              parseability_rejected_total: $sv_family_status_systemverilog_preprocessor_parseability_rejected_total,
              final_targets: $sv_family_status_systemverilog_preprocessor_final_targets,
              covered_reachable_rules: $sv_family_status_systemverilog_preprocessor_covered_reachable_rules,
              covered_reachable_branches: $sv_family_status_systemverilog_preprocessor_covered_reachable_branches,
              reachability_stage3_targets: $sv_family_status_systemverilog_preprocessor_reachability_stage3_targets,
              reachability_stage4_targets: $sv_family_status_systemverilog_preprocessor_reachability_stage4_targets
            },
            proof_surfaces: {
              syntax_closure_state_dir: $sv_family_status_systemverilog_preprocessor_syntax_closure_state_dir,
              syntax_closure_summary_txt: $sv_family_status_systemverilog_preprocessor_syntax_closure_summary_txt,
              syntax_closure_summary_json: $sv_family_status_systemverilog_preprocessor_syntax_closure_summary_json,
              aggregate_state_dir: $sv_family_status_systemverilog_preprocessor_aggregate_state_dir,
              aggregate_summary_txt: $sv_family_status_systemverilog_preprocessor_aggregate_summary_txt,
              aggregate_summary_json: $sv_family_status_systemverilog_preprocessor_aggregate_summary_json,
              reachability_state_dir: $sv_family_status_systemverilog_preprocessor_reachability_state_dir,
              reachability_summary_txt: $sv_family_status_systemverilog_preprocessor_reachability_summary_txt
            }
          }
        ]
      },
      family_status_contract: {
        gate: $sv_family_status_contract_gate,
        version: $sv_family_status_contract_gate_version,
        generated_at_utc: $sv_family_status_contract_generated_at_utc,
        family_status_state_dir: $sv_family_status_contract_family_status_state_dir,
        family_status_summary_json: $sv_family_status_contract_family_status_summary_json,
        family_status_summary_txt: $sv_family_status_contract_family_status_summary_txt,
        family_count: $sv_parser_family_status_contract_family_count,
        families: [
          {
            family: "systemverilog",
            tracker_alignment_ok: $sv_parser_family_status_contract_systemverilog_tracker_alignment_ok,
            false_criteria_count: $sv_parser_family_status_contract_systemverilog_false_criteria_count,
            unmet_details_count: $sv_parser_family_status_contract_systemverilog_unmet_details_count,
            primary_unmet_detail_criterion: $sv_parser_family_status_contract_systemverilog_primary_unmet_detail_criterion,
            unmet_closure_criteria: $sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_json,
            unmet_closure_criteria_details: $sv_parser_family_status_contract_systemverilog_unmet_closure_criteria_details_json,
            proof_surfaces: {
              parser_aggregate_state_dir: $sv_family_status_contract_systemverilog_parser_aggregate_state_dir,
              parser_aggregate_summary_txt: $sv_family_status_contract_systemverilog_parser_aggregate_summary_txt,
              parser_aggregate_summary_json: $sv_family_status_contract_systemverilog_parser_aggregate_summary_json,
              semantic_scope_contract_state_dir: $sv_family_status_contract_systemverilog_semantic_scope_contract_state_dir,
              semantic_scope_contract_summary_txt: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_txt,
              semantic_scope_contract_summary_json: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json
            }
          },
          {
            family: "systemverilog_preprocessor",
            tracker_alignment_ok: $sv_parser_family_status_contract_systemverilog_preprocessor_tracker_alignment_ok,
            false_criteria_count: $sv_parser_family_status_contract_systemverilog_preprocessor_false_criteria_count,
            unmet_details_count: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_details_count,
            primary_unmet_detail_criterion: $sv_parser_family_status_contract_systemverilog_preprocessor_primary_unmet_detail_criterion,
            unmet_closure_criteria: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_json,
            unmet_closure_criteria_details: $sv_parser_family_status_contract_systemverilog_preprocessor_unmet_closure_criteria_details_json,
            proof_surfaces: {
              aggregate_state_dir: $sv_family_status_contract_systemverilog_preprocessor_aggregate_state_dir,
              aggregate_summary_txt: $sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_txt,
              aggregate_summary_json: $sv_family_status_contract_systemverilog_preprocessor_aggregate_summary_json
            }
          }
        ]
      }
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
