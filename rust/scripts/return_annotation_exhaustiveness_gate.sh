#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_RETURN_ANNOTATION_EXHAUSTIVENESS_STATE_DIR:-$RUST_DIR/target/return_annotation_exhaustiveness_gate}"
ANNOTATION_STATE_DIR="$STATE_DIR/annotation_closed_loop"
PARITY_STATE_DIR="$STATE_DIR/stimuli_module_parity"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_JSON="$STATE_DIR/summary.json"
SUMMARY_TXT="$STATE_DIR/summary.txt"
PARITY_CONTRACT="${PGEN_RETURN_ANNOTATION_PARITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/return_annotation_stimuli_module_parity_contract.json}"

mkdir -p "$LOG_DIR" "$STATE_DIR"

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
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

assert_json() {
    local path="$1"
    local expr="$2"
    local message="$3"
    if ! jq -e "$expr" "$path" >/dev/null; then
        echo "error: $message (file: $path, expr: $expr)" >&2
        exit 1
    fi
}

extract_json_u64() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | numbers" "$path"
}

require_file "$PARITY_CONTRACT"

echo "==> Return annotation exhaustiveness gate"
echo "state_dir: $STATE_DIR"
echo "annotation_state_dir: $ANNOTATION_STATE_DIR"
echo "parity_state_dir: $PARITY_STATE_DIR"
echo "parity_contract: $PARITY_CONTRACT"

run_logged "build_generated_tools" \
    bash -lc "cd '$RUST_DIR' && cargo build --features generated_parsers --bin ast_pipeline --bin return_annotation_generated_audit"

run_logged "return_annotation_closed_loop" \
    env \
        PGEN_ANNOTATION_STIMULI_QUALITY_FILTER=return \
        PGEN_ANNOTATION_STIMULI_QUALITY_STATE_DIR="$ANNOTATION_STATE_DIR" \
        "$RUST_DIR/scripts/annotation_stimuli_quality_gate.sh"

RETURN_GAP_STAGE3="$ANNOTATION_STATE_DIR/work/return_gap_stage3.json"
RETURN_PARSEABILITY_REPORT="$ANNOTATION_STATE_DIR/work/return_parseability_report.json"
RETURN_SAMPLES_STAGE0="$ANNOTATION_STATE_DIR/work/return_samples_stage0.txt"
RETURN_SAMPLES_STAGE1="$ANNOTATION_STATE_DIR/work/return_samples_stage1.txt"
RETURN_SAMPLES_STAGE2="$ANNOTATION_STATE_DIR/work/return_samples_stage2.txt"
RETURN_SAMPLES_STAGE3="$ANNOTATION_STATE_DIR/work/return_samples_stage3.txt"

require_nonempty_file "$RETURN_GAP_STAGE3"
require_nonempty_file "$RETURN_PARSEABILITY_REPORT"
require_nonempty_file "$RETURN_SAMPLES_STAGE0"
require_nonempty_file "$RETURN_SAMPLES_STAGE1"
require_file "$RETURN_SAMPLES_STAGE2"
require_nonempty_file "$RETURN_SAMPLES_STAGE3"

assert_json "$RETURN_GAP_STAGE3" \
    '.grammar_name == "return_annotation" and (.targets | length) == 0' \
    "return stage3 gap report must close all actionable targets"
assert_json "$RETURN_GAP_STAGE3" \
    '.summary.covered_reachable_rules == .summary.reachable_rules and .summary.covered_reachable_branches == .summary.reachable_branches' \
    "return stage3 gap report must cover all reachable rules and branches"
assert_json "$RETURN_PARSEABILITY_REPORT" \
    '.grammar_name == "return_annotation" and .summary.accepted == .summary.attempts and .summary.rejected == 0 and .summary.parser_rejections == 0 and .summary.generation_errors == 0 and .summary.empty_generations == 0' \
    "return parseability report must show parser-backed acceptance with no failures"

run_logged "return_annotation_module_parity" \
    env \
        PGEN_STIMULI_MODULE_PARITY_STATE_DIR="$PARITY_STATE_DIR" \
        PGEN_STIMULI_MODULE_PARITY_CONTRACT="$PARITY_CONTRACT" \
        "$RUST_DIR/scripts/stimuli_module_parity_gate.sh"

PARITY_SAMPLES_INMEMORY="$PARITY_STATE_DIR/work/return_annotation_samples_inmemory.txt"
require_nonempty_file "$PARITY_SAMPLES_INMEMORY"

run_logged "return_annotation_generated_typed_ast_audit" \
    bash -lc "cd '$RUST_DIR' && cargo run --features generated_parsers --bin return_annotation_generated_audit -- \
        '$RETURN_SAMPLES_STAGE0' \
        '$RETURN_SAMPLES_STAGE1' \
        '$RETURN_SAMPLES_STAGE2' \
        '$RETURN_SAMPLES_STAGE3' \
        '$PARITY_SAMPLES_INMEMORY'"

INITIAL_TARGETS="$(extract_json_u64 "$ANNOTATION_STATE_DIR/work/return_gap_stage0.json" '.targets | length')"
FINAL_TARGETS="$(extract_json_u64 "$RETURN_GAP_STAGE3" '.targets | length')"
REACHABLE_RULES="$(extract_json_u64 "$RETURN_GAP_STAGE3" '.summary.reachable_rules')"
COVERED_REACHABLE_RULES="$(extract_json_u64 "$RETURN_GAP_STAGE3" '.summary.covered_reachable_rules')"
REACHABLE_BRANCHES="$(extract_json_u64 "$RETURN_GAP_STAGE3" '.summary.reachable_branches')"
COVERED_REACHABLE_BRANCHES="$(extract_json_u64 "$RETURN_GAP_STAGE3" '.summary.covered_reachable_branches')"
PARSEABILITY_ATTEMPTS="$(extract_json_u64 "$RETURN_PARSEABILITY_REPORT" '.summary.attempts')"
PARSEABILITY_ACCEPTED="$(extract_json_u64 "$RETURN_PARSEABILITY_REPORT" '.summary.accepted')"
ALTERNATE_ENTRY_ATTEMPTS="$(extract_json_u64 "$RETURN_PARSEABILITY_REPORT" '.target_drive_validation.alternate_entry_attempts_total')"

jq -n \
    --arg state_dir "$STATE_DIR" \
    --arg annotation_state_dir "$ANNOTATION_STATE_DIR" \
    --arg parity_state_dir "$PARITY_STATE_DIR" \
    --argjson initial_targets "$INITIAL_TARGETS" \
    --argjson final_targets "$FINAL_TARGETS" \
    --argjson reachable_rules "$REACHABLE_RULES" \
    --argjson covered_reachable_rules "$COVERED_REACHABLE_RULES" \
    --argjson reachable_branches "$REACHABLE_BRANCHES" \
    --argjson covered_reachable_branches "$COVERED_REACHABLE_BRANCHES" \
    --argjson parseability_attempts "$PARSEABILITY_ATTEMPTS" \
    --argjson parseability_accepted "$PARSEABILITY_ACCEPTED" \
    --argjson alternate_entry_attempts "$ALTERNATE_ENTRY_ATTEMPTS" \
    '{
        state_dir: $state_dir,
        annotation_state_dir: $annotation_state_dir,
        parity_state_dir: $parity_state_dir,
        return_annotation_closed_loop: {
            initial_targets: $initial_targets,
            final_targets: $final_targets,
            reachable_rules: $reachable_rules,
            covered_reachable_rules: $covered_reachable_rules,
            reachable_branches: $reachable_branches,
            covered_reachable_branches: $covered_reachable_branches
        },
        parseability: {
            attempts: $parseability_attempts,
            accepted: $parseability_accepted,
            alternate_entry_attempts: $alternate_entry_attempts
        }
    }' >"$SUMMARY_JSON"

{
    echo "PGEN Return Annotation Exhaustiveness Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "initial_targets: $INITIAL_TARGETS"
    echo "final_targets: $FINAL_TARGETS"
    echo "reachable_rules: $REACHABLE_RULES"
    echo "covered_reachable_rules: $COVERED_REACHABLE_RULES"
    echo "reachable_branches: $REACHABLE_BRANCHES"
    echo "covered_reachable_branches: $COVERED_REACHABLE_BRANCHES"
    echo "parseability_attempts: $PARSEABILITY_ATTEMPTS"
    echo "parseability_accepted: $PARSEABILITY_ACCEPTED"
    echo "alternate_entry_attempts: $ALTERNATE_ENTRY_ATTEMPTS"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ Return annotation exhaustiveness gate passed.
Logs: $LOG_DIR
Artifacts: $STATE_DIR
EOF
