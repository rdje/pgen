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
sv_failure_summary_txt="$sota_state_dir/work/sv_failure_context_contract_gate/summary.txt"
sv_roundtrip_summary_txt="$sota_state_dir/work/sv_roundtrip_contract_gate/summary.txt"

require_nonempty_file "$sota_summary_txt"
require_nonempty_file "$sv_failure_summary_txt"
require_nonempty_file "$sv_roundtrip_summary_txt"

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

sv_failure_generation_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_generation_failure_context_excerpts")"
sv_failure_shadow_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_shadow_failure_context_excerpts")"
sv_roundtrip_initial_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_targets")"
sv_roundtrip_replay_targets="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_targets")"
sv_roundtrip_initial_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_covered_reachable_rules")"
sv_roundtrip_replay_rules="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_covered_reachable_rules")"
sv_roundtrip_initial_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_initial_covered_reachable_branches")"
sv_roundtrip_replay_branches="$(extract_summary_value "$sv_roundtrip_summary_txt" "systemverilog_roundtrip_replay_covered_reachable_branches")"

svpp_failure_excerpts="$(extract_summary_value "$sv_failure_summary_txt" "systemverilog_preprocessor_failure_context_excerpts")"
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

{
    echo "SV Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
    echo "existing_sota_exit_state_dir: ${EXISTING_SOTA_EXIT_STATE_DIR:-<unset>}"
    echo "sota_exit_state_dir: $sota_state_dir"
    echo "sota_exit_summary_txt: $sota_summary_txt"
    echo "sv_failure_context_contract_summary_txt: $sv_failure_summary_txt"
    echo "sv_roundtrip_contract_summary_txt: $sv_roundtrip_summary_txt"
    echo "sv_failure_context_generation_excerpts: $sv_failure_generation_excerpts"
    echo "sv_failure_context_shadow_excerpts: $sv_failure_shadow_excerpts"
    echo "sv_roundtrip_initial_targets: $sv_roundtrip_initial_targets"
    echo "sv_roundtrip_replay_targets: $sv_roundtrip_replay_targets"
    echo "sv_roundtrip_initial_covered_reachable_rules: $sv_roundtrip_initial_rules"
    echo "sv_roundtrip_replay_covered_reachable_rules: $sv_roundtrip_replay_rules"
    echo "sv_roundtrip_initial_covered_reachable_branches: $sv_roundtrip_initial_branches"
    echo "sv_roundtrip_replay_covered_reachable_branches: $sv_roundtrip_replay_branches"
    echo "sv_preprocessor_failure_context_excerpts: $svpp_failure_excerpts"
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
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
