#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_FAMILY_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_parser_family_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

FRONTEND_GATE="$RUST_DIR/scripts/ebnf_frontend_readiness_gate.sh"
DUAL_RUN_GATE="$RUST_DIR/scripts/ebnf_frontend_dual_run_diff_gate.sh"
STIMULI_GATE="$RUST_DIR/scripts/ebnf_stimuli_quality_gate.sh"

EXISTING_FRONTEND_STATE_DIR="${PGEN_REGEX_FAMILY_CONTRACT_EXISTING_FRONTEND_STATE_DIR:-}"
EXISTING_DUAL_RUN_STATE_DIR="${PGEN_REGEX_FAMILY_CONTRACT_EXISTING_DUAL_RUN_STATE_DIR:-}"
EXISTING_STIMULI_STATE_DIR="${PGEN_REGEX_FAMILY_CONTRACT_EXISTING_STIMULI_STATE_DIR:-}"

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

extract_csv_value() {
    local path="$1"
    local grammar="$2"
    local field="$3"
    python3 - "$path" "$grammar" "$field" <<'PY'
import csv
import sys

path, grammar, field = sys.argv[1:]
with open(path, newline="") as fh:
    rows = list(csv.DictReader(fh))

for row in rows:
    if row.get("grammar") == grammar:
        if field not in row:
            raise SystemExit(f"missing field '{field}' in csv '{path}'")
        print(row[field])
        raise SystemExit(0)

raise SystemExit(f"missing grammar '{grammar}' in csv '{path}'")
PY
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

assert_int_ge() {
    local label="$1"
    local actual="$2"
    local minimum="$3"
    if (( actual < minimum )); then
        echo "error: ${label} expected >= ${minimum} but found ${actual}" >&2
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
require_tool python3
require_file "$FRONTEND_GATE"
require_file "$DUAL_RUN_GATE"
require_file "$STIMULI_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

frontend_state_dir="${EXISTING_FRONTEND_STATE_DIR:-$WORK_DIR/ebnf_frontend_gate}"
dual_run_state_dir="${EXISTING_DUAL_RUN_STATE_DIR:-$WORK_DIR/ebnf_frontend_dual_run_gate}"
stimuli_state_dir="${EXISTING_STIMULI_STATE_DIR:-$WORK_DIR/ebnf_stimuli_quality_gate}"

if [[ -z "$EXISTING_FRONTEND_STATE_DIR" ]]; then
    run_logged "ebnf_frontend_gate" env \
        PGEN_EBNF_FRONTEND_STATE_DIR="$frontend_state_dir" \
        PGEN_EBNF_FRONTEND_STRICT=1 \
        "$FRONTEND_GATE"
fi

if [[ -z "$EXISTING_DUAL_RUN_STATE_DIR" ]]; then
    run_logged "ebnf_frontend_dual_run_gate" env \
        PGEN_EBNF_DUAL_RUN_STATE_DIR="$dual_run_state_dir" \
        PGEN_EBNF_DUAL_RUN_STRICT=1 \
        "$DUAL_RUN_GATE"
fi

if [[ -z "$EXISTING_STIMULI_STATE_DIR" ]]; then
    run_logged "ebnf_stimuli_quality_gate" env \
        PGEN_EBNF_STIMULI_QUALITY_STATE_DIR="$stimuli_state_dir" \
        "$STIMULI_GATE"
fi

frontend_summary_txt="$frontend_state_dir/summary.txt"
frontend_summary_csv="$frontend_state_dir/summary.csv"
dual_run_summary_txt="$dual_run_state_dir/summary.txt"
dual_run_summary_csv="$dual_run_state_dir/summary.csv"
dual_run_summary_json="$dual_run_state_dir/summary.json"
stimuli_summary_txt="$stimuli_state_dir/summary.txt"
stimuli_summary_csv="$stimuli_state_dir/summary.csv"

require_nonempty_file "$frontend_summary_txt"
require_nonempty_file "$frontend_summary_csv"
require_nonempty_file "$dual_run_summary_txt"
require_nonempty_file "$dual_run_summary_csv"
require_nonempty_file "$dual_run_summary_json"
require_nonempty_file "$stimuli_summary_txt"
require_nonempty_file "$stimuli_summary_csv"

frontend_strict_mode="$(extract_summary_value "$frontend_summary_txt" "strict_mode")"
frontend_impl="$(extract_summary_value "$frontend_summary_txt" "frontend_impl")"
frontend_regex_ebnf_to_json="$(extract_csv_value "$frontend_summary_csv" "regex" "ebnf_to_json")"
frontend_regex_json_to_parser="$(extract_csv_value "$frontend_summary_csv" "regex" "json_to_parser")"
frontend_regex_json_to_stimuli="$(extract_csv_value "$frontend_summary_csv" "regex" "json_to_stimuli")"
frontend_regex_overall="$(extract_csv_value "$frontend_summary_csv" "regex" "overall")"
frontend_regex_notes="$(extract_csv_value "$frontend_summary_csv" "regex" "notes")"

dual_run_strict_mode="$(extract_summary_value "$dual_run_summary_txt" "strict_mode")"
dual_run_regex_perl_ebnf_to_json="$(jq -r '.entries[] | select(.grammar=="regex") | .perl_ebnf_to_json' "$dual_run_summary_json")"
dual_run_regex_rust_parse="$(jq -r '.entries[] | select(.grammar=="regex") | .rust_parse' "$dual_run_summary_json")"
dual_run_regex_rust_parse_full="$(jq -r '.entries[] | select(.grammar=="regex") | .rust_parse_full' "$dual_run_summary_json")"
dual_run_regex_perl_rule_count="$(jq -r '.entries[] | select(.grammar=="regex") | .perl_rule_count' "$dual_run_summary_json")"
dual_run_regex_rust_rule_count="$(jq -r '.entries[] | select(.grammar=="regex") | .rust_rule_count' "$dual_run_summary_json")"
dual_run_regex_raw_ast_status="$(jq -r '.entries[] | select(.grammar=="regex") | .raw_ast_status' "$dual_run_summary_json")"
dual_run_regex_raw_ast_missing_on_perl_count="$(jq -r '.entries[] | select(.grammar=="regex") | .raw_ast_missing_on_perl_count' "$dual_run_summary_json")"
dual_run_regex_raw_ast_missing_on_rust_count="$(jq -r '.entries[] | select(.grammar=="regex") | .raw_ast_missing_on_rust_count' "$dual_run_summary_json")"
dual_run_regex_consumed_pct="$(jq -r '.entries[] | select(.grammar=="regex") | .consumed_pct' "$dual_run_summary_json")"
dual_run_regex_overall="$(jq -r '.entries[] | select(.grammar=="regex") | .overall' "$dual_run_summary_json")"
dual_run_regex_notes="$(jq -r '.entries[] | select(.grammar=="regex") | .notes' "$dual_run_summary_json")"

stimuli_contract_file="$(extract_summary_value "$stimuli_summary_txt" "contract_file")"
stimuli_regex_grammar_name="$(extract_csv_value "$stimuli_summary_csv" "regex" "grammar_name")"
stimuli_regex_parseability_required="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_required")"
stimuli_regex_initial_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "initial_targets")"
stimuli_regex_resolved_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "resolved_targets")"
stimuli_regex_final_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "final_targets")"
stimuli_regex_target_attempts="$(extract_csv_value "$stimuli_summary_csv" "regex" "target_attempts")"
stimuli_regex_stage0_successes="$(extract_csv_value "$stimuli_summary_csv" "regex" "stage0_successes")"
stimuli_regex_stage3_successes="$(extract_csv_value "$stimuli_summary_csv" "regex" "stage3_successes")"
stimuli_regex_status="$(extract_csv_value "$stimuli_summary_csv" "regex" "status")"

assert_equal "frontend regex ebnf_to_json" "pass" "$frontend_regex_ebnf_to_json"
assert_equal "frontend regex json_to_parser" "pass" "$frontend_regex_json_to_parser"
assert_equal "frontend regex json_to_stimuli" "pass" "$frontend_regex_json_to_stimuli"
assert_equal "frontend regex overall" "pass" "$frontend_regex_overall"

assert_equal "dual-run regex perl_ebnf_to_json" "pass" "$dual_run_regex_perl_ebnf_to_json"
assert_equal "dual-run regex rust_parse" "pass" "$dual_run_regex_rust_parse"
assert_equal "dual-run regex rust_parse_full" "pass" "$dual_run_regex_rust_parse_full"
assert_equal "dual-run regex overall" "pass" "$dual_run_regex_overall"
assert_equal "dual-run regex raw_ast_missing_on_rust_count" "0" "$dual_run_regex_raw_ast_missing_on_rust_count"
case "$dual_run_regex_raw_ast_status" in
    parity|perl_under_reports)
        ;;
    *)
        echo "error: dual-run regex raw_ast_status must be 'parity' or 'perl_under_reports' but found '$dual_run_regex_raw_ast_status'" >&2
        exit 1
        ;;
esac
if (( dual_run_regex_rust_rule_count < dual_run_regex_perl_rule_count )); then
    echo "error: dual-run regex rust_rule_count ${dual_run_regex_rust_rule_count} is below perl_rule_count ${dual_run_regex_perl_rule_count}" >&2
    exit 1
fi

assert_equal "stimuli regex grammar_name" "regex" "$stimuli_regex_grammar_name"
assert_equal "stimuli regex parseability_required" "0" "$stimuli_regex_parseability_required"
assert_equal "stimuli regex status" "pass" "$stimuli_regex_status"
assert_int_ge "stimuli regex initial_targets" "$stimuli_regex_initial_targets" 1
assert_int_ge "stimuli regex resolved_targets" "$stimuli_regex_resolved_targets" 1
assert_int_ge "stimuli regex target_attempts" "$stimuli_regex_target_attempts" 1
assert_int_ge "stimuli regex stage0_successes" "$stimuli_regex_stage0_successes" 1
if (( stimuli_regex_resolved_targets + stimuli_regex_final_targets != stimuli_regex_initial_targets )); then
    echo "error: stimuli regex target accounting mismatch (${stimuli_regex_resolved_targets} + ${stimuli_regex_final_targets} != ${stimuli_regex_initial_targets})" >&2
    exit 1
fi
if (( stimuli_regex_stage3_successes < stimuli_regex_stage0_successes )); then
    echo "error: stimuli regex stage3_successes ${stimuli_regex_stage3_successes} regressed below stage0_successes ${stimuli_regex_stage0_successes}" >&2
    exit 1
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "Regex Parser Family Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "frontend_state_dir: $frontend_state_dir"
    echo "frontend_summary_txt: $frontend_summary_txt"
    echo "frontend_summary_csv: $frontend_summary_csv"
    echo "frontend_impl: $frontend_impl"
    echo "frontend_regex_ebnf_to_json: $frontend_regex_ebnf_to_json"
    echo "frontend_regex_json_to_parser: $frontend_regex_json_to_parser"
    echo "frontend_regex_json_to_stimuli: $frontend_regex_json_to_stimuli"
    echo "frontend_regex_overall: $frontend_regex_overall"
    echo "frontend_regex_notes: $frontend_regex_notes"
    echo "dual_run_state_dir: $dual_run_state_dir"
    echo "dual_run_summary_txt: $dual_run_summary_txt"
    echo "dual_run_summary_csv: $dual_run_summary_csv"
    echo "dual_run_summary_json: $dual_run_summary_json"
    echo "dual_run_regex_perl_ebnf_to_json: $dual_run_regex_perl_ebnf_to_json"
    echo "dual_run_regex_rust_parse: $dual_run_regex_rust_parse"
    echo "dual_run_regex_rust_parse_full: $dual_run_regex_rust_parse_full"
    echo "dual_run_regex_perl_rule_count: $dual_run_regex_perl_rule_count"
    echo "dual_run_regex_rust_rule_count: $dual_run_regex_rust_rule_count"
    echo "dual_run_regex_raw_ast_status: $dual_run_regex_raw_ast_status"
    echo "dual_run_regex_raw_ast_missing_on_perl_count: $dual_run_regex_raw_ast_missing_on_perl_count"
    echo "dual_run_regex_raw_ast_missing_on_rust_count: $dual_run_regex_raw_ast_missing_on_rust_count"
    echo "dual_run_regex_consumed_pct: $dual_run_regex_consumed_pct"
    echo "dual_run_regex_overall: $dual_run_regex_overall"
    echo "dual_run_regex_notes: $dual_run_regex_notes"
    echo "stimuli_state_dir: $stimuli_state_dir"
    echo "stimuli_summary_txt: $stimuli_summary_txt"
    echo "stimuli_summary_csv: $stimuli_summary_csv"
    echo "stimuli_contract_file: $stimuli_contract_file"
    echo "stimuli_regex_parseability_required: $stimuli_regex_parseability_required"
    echo "stimuli_regex_initial_targets: $stimuli_regex_initial_targets"
    echo "stimuli_regex_resolved_targets: $stimuli_regex_resolved_targets"
    echo "stimuli_regex_final_targets: $stimuli_regex_final_targets"
    echo "stimuli_regex_target_attempts: $stimuli_regex_target_attempts"
    echo "stimuli_regex_stage0_successes: $stimuli_regex_stage0_successes"
    echo "stimuli_regex_stage3_successes: $stimuli_regex_stage3_successes"
    echo "stimuli_regex_status: $stimuli_regex_status"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "regex_parser_family_contract_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg frontend_state_dir "$frontend_state_dir" \
    --arg frontend_summary_txt "$frontend_summary_txt" \
    --arg frontend_summary_csv "$frontend_summary_csv" \
    --arg frontend_impl "$frontend_impl" \
    --arg frontend_regex_ebnf_to_json "$frontend_regex_ebnf_to_json" \
    --arg frontend_regex_json_to_parser "$frontend_regex_json_to_parser" \
    --arg frontend_regex_json_to_stimuli "$frontend_regex_json_to_stimuli" \
    --arg frontend_regex_overall "$frontend_regex_overall" \
    --arg frontend_regex_notes "$frontend_regex_notes" \
    --arg dual_run_state_dir "$dual_run_state_dir" \
    --arg dual_run_summary_txt "$dual_run_summary_txt" \
    --arg dual_run_summary_csv "$dual_run_summary_csv" \
    --arg dual_run_summary_json "$dual_run_summary_json" \
    --arg dual_run_regex_perl_ebnf_to_json "$dual_run_regex_perl_ebnf_to_json" \
    --arg dual_run_regex_rust_parse "$dual_run_regex_rust_parse" \
    --arg dual_run_regex_rust_parse_full "$dual_run_regex_rust_parse_full" \
    --argjson dual_run_regex_perl_rule_count "$dual_run_regex_perl_rule_count" \
    --argjson dual_run_regex_rust_rule_count "$dual_run_regex_rust_rule_count" \
    --arg dual_run_regex_raw_ast_status "$dual_run_regex_raw_ast_status" \
    --argjson dual_run_regex_raw_ast_missing_on_perl_count "$dual_run_regex_raw_ast_missing_on_perl_count" \
    --argjson dual_run_regex_raw_ast_missing_on_rust_count "$dual_run_regex_raw_ast_missing_on_rust_count" \
    --argjson dual_run_regex_consumed_pct "$dual_run_regex_consumed_pct" \
    --arg dual_run_regex_overall "$dual_run_regex_overall" \
    --arg dual_run_regex_notes "$dual_run_regex_notes" \
    --arg stimuli_state_dir "$stimuli_state_dir" \
    --arg stimuli_summary_txt "$stimuli_summary_txt" \
    --arg stimuli_summary_csv "$stimuli_summary_csv" \
    --arg stimuli_contract_file "$stimuli_contract_file" \
    --argjson stimuli_regex_parseability_required "$stimuli_regex_parseability_required" \
    --argjson stimuli_regex_initial_targets "$stimuli_regex_initial_targets" \
    --argjson stimuli_regex_resolved_targets "$stimuli_regex_resolved_targets" \
    --argjson stimuli_regex_final_targets "$stimuli_regex_final_targets" \
    --argjson stimuli_regex_target_attempts "$stimuli_regex_target_attempts" \
    --argjson stimuli_regex_stage0_successes "$stimuli_regex_stage0_successes" \
    --argjson stimuli_regex_stage3_successes "$stimuli_regex_stage3_successes" \
    --arg stimuli_regex_status "$stimuli_regex_status" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      proof_surfaces: {
        frontend_state_dir: $frontend_state_dir,
        frontend_summary_txt: $frontend_summary_txt,
        frontend_summary_csv: $frontend_summary_csv,
        dual_run_state_dir: $dual_run_state_dir,
        dual_run_summary_txt: $dual_run_summary_txt,
        dual_run_summary_csv: $dual_run_summary_csv,
        dual_run_summary_json: $dual_run_summary_json,
        stimuli_state_dir: $stimuli_state_dir,
        stimuli_summary_txt: $stimuli_summary_txt,
        stimuli_summary_csv: $stimuli_summary_csv
      },
      metrics: {
        frontend_impl: $frontend_impl,
        frontend_regex_ebnf_to_json: $frontend_regex_ebnf_to_json,
        frontend_regex_json_to_parser: $frontend_regex_json_to_parser,
        frontend_regex_json_to_stimuli: $frontend_regex_json_to_stimuli,
        frontend_regex_overall: $frontend_regex_overall,
        frontend_regex_notes: $frontend_regex_notes,
        dual_run_regex_perl_ebnf_to_json: $dual_run_regex_perl_ebnf_to_json,
        dual_run_regex_rust_parse: $dual_run_regex_rust_parse,
        dual_run_regex_rust_parse_full: $dual_run_regex_rust_parse_full,
        dual_run_regex_perl_rule_count: $dual_run_regex_perl_rule_count,
        dual_run_regex_rust_rule_count: $dual_run_regex_rust_rule_count,
        dual_run_regex_raw_ast_status: $dual_run_regex_raw_ast_status,
        dual_run_regex_raw_ast_missing_on_perl_count: $dual_run_regex_raw_ast_missing_on_perl_count,
        dual_run_regex_raw_ast_missing_on_rust_count: $dual_run_regex_raw_ast_missing_on_rust_count,
        dual_run_regex_consumed_pct: $dual_run_regex_consumed_pct,
        dual_run_regex_overall: $dual_run_regex_overall,
        dual_run_regex_notes: $dual_run_regex_notes,
        stimuli_contract_file: $stimuli_contract_file,
        stimuli_regex_parseability_required: $stimuli_regex_parseability_required,
        stimuli_regex_initial_targets: $stimuli_regex_initial_targets,
        stimuli_regex_resolved_targets: $stimuli_regex_resolved_targets,
        stimuli_regex_final_targets: $stimuli_regex_final_targets,
        stimuli_regex_target_attempts: $stimuli_regex_target_attempts,
        stimuli_regex_stage0_successes: $stimuli_regex_stage0_successes,
        stimuli_regex_stage3_successes: $stimuli_regex_stage3_successes,
        stimuli_regex_status: $stimuli_regex_status
      }
    }' >"$SUMMARY_JSON"

echo "✅ Regex parser-family contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
