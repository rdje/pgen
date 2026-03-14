#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_parser_aggregate_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

BASE_CONTRACT_FILE="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json}"
SV_GATE_SCRIPT="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"
EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"

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
    local name="$1"
    shift
    local log_file="$LOG_DIR/${name}.log"
    echo "==> $name"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok ($log_file)"
    else
        echo "error: stage '$name' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | numbers" "$path"
}

assert_same_json() {
    local left_path="$1"
    local right_path="$2"
    local label="$3"
    if ! jq -es '.[0] == .[1]' "$left_path" "$right_path" >/dev/null; then
        echo "error: JSON mismatch for ${label}: '$left_path' vs '$right_path'" >&2
        exit 1
    fi
}

require_tool jq
if [[ -z "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    require_file "$BASE_CONTRACT_FILE"
    require_file "$SV_GATE_SCRIPT"
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

generation_contract="<existing_artifact_mode>"
shadow_contract="<existing_artifact_mode>"
generation_state_dir="<existing_artifact_mode>"
shadow_state_dir="<existing_artifact_mode>"

if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    generation_report_json="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR/work/systemverilog_parseability_generation_report.json"
    shadow_report_json="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR/work/systemverilog_closed_loop_parseability_shadow_report.json"
else
    generation_contract="$WORK_DIR/systemverilog_generation_only_contract.json"
    shadow_contract="$WORK_DIR/systemverilog_shadow_enabled_contract.json"

    jq '
        .closed_loop.enabled = true
        | .closed_loop.parseability_shadow_enabled = false
        | .nexsim_realistic_corpus.enforce = false
        | .performance_budgets.enforce = false
        | .parse_full_quality.enforce_min_pass_ratio = false
    ' "$BASE_CONTRACT_FILE" >"$generation_contract"

    jq '
        .closed_loop.enabled = true
        | .closed_loop.parseability_shadow_enabled = true
        | .nexsim_realistic_corpus.enforce = false
        | .performance_budgets.enforce = false
        | .parse_full_quality.enforce_min_pass_ratio = false
    ' "$BASE_CONTRACT_FILE" >"$shadow_contract"

    generation_state_dir="$WORK_DIR/generation_state"
    shadow_state_dir="$WORK_DIR/shadow_state"

    run_logged "generation_only_aggregate_probe" \
        env \
            PGEN_SV_STIMULI_QUALITY_CONTRACT="$generation_contract" \
            PGEN_SV_STIMULI_QUALITY_STATE_DIR="$generation_state_dir" \
            PGEN_SV_STIMULI_QUALITY_COUNT=1 \
            PGEN_SV_STIMULI_QUALITY_LRM_PROFILES=2017 \
            PGEN_SV_STIMULI_DIFF_MODE=0 \
            PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
            PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
            "$SV_GATE_SCRIPT"

    run_logged "shadow_enabled_aggregate_probe" \
        env \
            PGEN_SV_STIMULI_QUALITY_CONTRACT="$shadow_contract" \
            PGEN_SV_STIMULI_QUALITY_STATE_DIR="$shadow_state_dir" \
            PGEN_SV_STIMULI_QUALITY_COUNT=1 \
            PGEN_SV_STIMULI_QUALITY_LRM_PROFILES=2017 \
            PGEN_SV_STIMULI_DIFF_MODE=0 \
            PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
            PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
            "$SV_GATE_SCRIPT"

    generation_report_json="$generation_state_dir/work/systemverilog_parseability_generation_report.json"
    shadow_report_json="$shadow_state_dir/work/systemverilog_closed_loop_parseability_shadow_report.json"
fi

require_nonempty_file "$generation_report_json"
require_nonempty_file "$shadow_report_json"

closed_loop_initial_coverage_json="$(dirname "$shadow_report_json")/profile_2017_initial_coverage.json"
closed_loop_initial_replay_coverage_json="$(dirname "$shadow_report_json")/profile_2017_initial_replay_coverage.json"
closed_loop_initial_gap_json="$(dirname "$shadow_report_json")/profile_2017_initial_gap.json"
closed_loop_initial_replay_gap_json="$(dirname "$shadow_report_json")/profile_2017_initial_replay_gap.json"
closed_loop_replay_gap_json="$(dirname "$shadow_report_json")/profile_2017_replay_gap.json"

require_nonempty_file "$closed_loop_initial_coverage_json"
require_nonempty_file "$closed_loop_initial_replay_coverage_json"
require_nonempty_file "$closed_loop_initial_gap_json"
require_nonempty_file "$closed_loop_initial_replay_gap_json"
require_nonempty_file "$closed_loop_replay_gap_json"

assert_same_json "$closed_loop_initial_coverage_json" "$closed_loop_initial_replay_coverage_json" "sv parser aggregate initial coverage replay"
assert_same_json "$closed_loop_initial_gap_json" "$closed_loop_initial_replay_gap_json" "sv parser aggregate initial gap replay"

if ! jq -e '
    .grammar_name == "systemverilog"
    and .enabled == true
    and (.counterexamples | type == "array")
    and ((.counterexamples | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
        and has("parser_error")
        and has("failure_position")
        and has("failure_line")
        and has("failure_column")
        and has("profile")
        and has("sample_index")
        and has("seed")
    )
' "$generation_report_json" >/dev/null; then
    echo "error: generation aggregate report contract failed: $generation_report_json" >&2
    cat "$generation_report_json" >&2
    exit 1
fi

if ! jq -e '
    .grammar_name == "systemverilog"
    and .enabled == true
    and .effective_mode == "enabled"
    and (.counterexamples | type == "array")
    and (.counterexamples_captured_total | numbers) >= (.counterexamples | length)
    and ((.counterexamples | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and ((.profiles | length) >= 1)
    and all(.profiles[]; (.counterexamples_captured | numbers) >= 0)
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
        and has("parser_error")
        and has("failure_position")
        and has("failure_line")
        and has("failure_column")
        and has("profile")
    )
' "$shadow_report_json" >/dev/null; then
    echo "error: replay-shadow aggregate report contract failed: $shadow_report_json" >&2
    cat "$shadow_report_json" >&2
    exit 1
fi

if ! jq -e '
    (.observed.attempts_total | numbers)
        == (
            (.observed.accepted_total | numbers)
            + (.observed.rejected_total | numbers)
            + (.observed.generation_errors_total | numbers)
            + (.observed.empty_generations_total | numbers)
        )
    and (.target_drive_validation.primary_entry_attempts_total | numbers) == (.observed.attempts_total | numbers)
    and (.target_drive_validation.primary_entry_accepted_outputs_total | numbers) == (.observed.accepted_total | numbers)
    and (.target_drive_validation.primary_entry_rejected_outputs_total | numbers) == (.observed.rejected_total | numbers)
    and (
        (.target_drive_validation.alternate_entry_attempts_total | numbers)
        == (
            (.target_drive_validation.alternate_entry_accepted_outputs_total | numbers)
            + (.target_drive_validation.alternate_entry_rejected_outputs_total | numbers)
        )
    )
' "$shadow_report_json" >/dev/null; then
    echo "error: replay-shadow aggregate report totals are internally inconsistent: $shadow_report_json" >&2
    cat "$shadow_report_json" >&2
    exit 1
fi

initial_target_count="$(extract_json_number "$closed_loop_initial_gap_json" '((.targets // []) | length)')"
replay_target_count="$(extract_json_number "$closed_loop_replay_gap_json" '((.targets // []) | length)')"
initial_covered_reachable_rules="$(extract_json_number "$closed_loop_initial_gap_json" '.summary.covered_reachable_rules')"
replay_covered_reachable_rules="$(extract_json_number "$closed_loop_replay_gap_json" '.summary.covered_reachable_rules')"
initial_reachable_rules="$(extract_json_number "$closed_loop_initial_gap_json" '.summary.reachable_rules')"
replay_reachable_rules="$(extract_json_number "$closed_loop_replay_gap_json" '.summary.reachable_rules')"
initial_covered_reachable_branches="$(extract_json_number "$closed_loop_initial_gap_json" '.summary.covered_reachable_branches')"
replay_covered_reachable_branches="$(extract_json_number "$closed_loop_replay_gap_json" '.summary.covered_reachable_branches')"
initial_reachable_branches="$(extract_json_number "$closed_loop_initial_gap_json" '.summary.reachable_branches')"
replay_reachable_branches="$(extract_json_number "$closed_loop_replay_gap_json" '.summary.reachable_branches')"

generation_counterexample_triage_json="$WORK_DIR/systemverilog_parseability_generation_counterexample_triage.json"
generation_counterexample_triage_txt="$WORK_DIR/systemverilog_parseability_generation_counterexample_triage.txt"
shadow_counterexample_triage_json="$WORK_DIR/systemverilog_closed_loop_parseability_shadow_counterexample_triage.json"
shadow_counterexample_triage_txt="$WORK_DIR/systemverilog_closed_loop_parseability_shadow_counterexample_triage.txt"

if (( replay_target_count > initial_target_count )); then
    echo "error: replay target debt increased: initial=$initial_target_count replay=$replay_target_count" >&2
    exit 1
fi

if (( replay_covered_reachable_rules < initial_covered_reachable_rules )); then
    echo "error: replay reachable-rule coverage regressed: initial=$initial_covered_reachable_rules replay=$replay_covered_reachable_rules" >&2
    exit 1
fi

if (( replay_covered_reachable_branches < initial_covered_reachable_branches )); then
    echo "error: replay reachable-branch coverage regressed: initial=$initial_covered_reachable_branches replay=$replay_covered_reachable_branches" >&2
    exit 1
fi

if (( replay_reachable_rules != initial_reachable_rules )); then
    echo "error: reachable-rule universe drifted across focused replay: initial=$initial_reachable_rules replay=$replay_reachable_rules" >&2
    exit 1
fi

if (( replay_reachable_branches != initial_reachable_branches )); then
    echo "error: reachable-branch universe drifted across focused replay: initial=$initial_reachable_branches replay=$replay_reachable_branches" >&2
    exit 1
fi

jq '
    {
        grammar_name: .grammar_name,
        aggregate_surface: "generation",
        total_counterexamples: ((.counterexamples // []) | length),
        by_stage: (
            (.counterexamples // [])
            | sort_by(.stage)
            | group_by(.stage)
            | map({
                stage: .[0].stage,
                count: length
            })
        ),
        by_parser_error: (
            (.counterexamples // [])
            | sort_by(.parser_error)
            | group_by(.parser_error)
            | map({
                parser_error: .[0].parser_error,
                count: length
            })
        ),
        by_shrunk_sample: (
            (.counterexamples // [])
            | sort_by(.shrunk_sample)
            | group_by(.shrunk_sample)
            | map({
                shrunk_sample: .[0].shrunk_sample,
                count: length
            })
        ),
        by_failure_location: (
            (.counterexamples // [])
            | sort_by(.failure_line, .failure_column)
            | group_by({failure_line, failure_column})
            | map({
                failure_line: .[0].failure_line,
                failure_column: .[0].failure_column,
                count: length
            })
        ),
        sample_previews: (
            (.counterexamples // [])[:5]
            | map({
                stage,
                parser_error,
                failure_line,
                failure_column,
                profile,
                sample_index,
                seed,
                shrunk_sample,
                sample_preview: (.sample[:80])
            })
        )
    }
' "$generation_report_json" >"$generation_counterexample_triage_json"
require_nonempty_file "$generation_counterexample_triage_json"

{
    echo "SV Parser Generation Counterexample Triage"
    echo "source_report: $generation_report_json"
    jq -r '.by_stage[]? | "stage_count[\(.stage)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_shrunk_sample[]? | "shrunk_sample_count[\(.shrunk_sample | @json)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$generation_counterexample_triage_json"
} >"$generation_counterexample_triage_txt"
require_nonempty_file "$generation_counterexample_triage_txt"

jq '
    {
        grammar_name: .grammar_name,
        aggregate_surface: "replay_shadow",
        total_counterexamples: ((.counterexamples // []) | length),
        by_stage: (
            (.counterexamples // [])
            | sort_by(.stage)
            | group_by(.stage)
            | map({
                stage: .[0].stage,
                count: length
            })
        ),
        by_parser_error: (
            (.counterexamples // [])
            | sort_by(.parser_error)
            | group_by(.parser_error)
            | map({
                parser_error: .[0].parser_error,
                count: length
            })
        ),
        by_shrunk_sample: (
            (.counterexamples // [])
            | sort_by(.shrunk_sample)
            | group_by(.shrunk_sample)
            | map({
                shrunk_sample: .[0].shrunk_sample,
                count: length
            })
        ),
        by_failure_location: (
            (.counterexamples // [])
            | sort_by(.failure_line, .failure_column)
            | group_by({failure_line, failure_column})
            | map({
                failure_line: .[0].failure_line,
                failure_column: .[0].failure_column,
                count: length
            })
        ),
        sample_previews: (
            (.counterexamples // [])[:5]
            | map({
                stage,
                parser_error,
                failure_line,
                failure_column,
                profile,
                shrunk_sample,
                sample_preview: (.sample[:80])
            })
        )
    }
' "$shadow_report_json" >"$shadow_counterexample_triage_json"
require_nonempty_file "$shadow_counterexample_triage_json"

{
    echo "SV Parser Replay-Shadow Counterexample Triage"
    echo "source_report: $shadow_report_json"
    jq -r '.by_stage[]? | "stage_count[\(.stage)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_shrunk_sample[]? | "shrunk_sample_count[\(.shrunk_sample | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$shadow_counterexample_triage_json"
} >"$shadow_counterexample_triage_txt"
require_nonempty_file "$shadow_counterexample_triage_txt"

generation_parser_rejections="$(extract_json_number "$generation_report_json" '.observed.parser_rejections_total')"
generation_counterexamples_count="$(extract_json_number "$generation_report_json" '((.counterexamples // []) | length)')"
shadow_parser_rejections="$(extract_json_number "$shadow_report_json" '.observed.parser_rejections_total')"
shadow_counterexamples_count="$(extract_json_number "$shadow_report_json" '((.counterexamples // []) | length)')"
shadow_counterexamples_captured_total="$(extract_json_number "$shadow_report_json" '.counterexamples_captured_total')"
generation_counterexample_unique_shrunk_samples="$(extract_json_number "$generation_counterexample_triage_json" '(.by_shrunk_sample | length)')"
generation_counterexample_unique_failure_locations="$(extract_json_number "$generation_counterexample_triage_json" '(.by_failure_location | length)')"
shadow_counterexample_unique_shrunk_samples="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_shrunk_sample | length)')"
shadow_counterexample_unique_failure_locations="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_failure_location | length)')"

{
    echo "SV Parser Aggregate Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "existing_sv_stimuli_quality_state_dir: ${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-<unset>}"
    echo "base_contract_file: $BASE_CONTRACT_FILE"
    echo "generation_contract_file: $generation_contract"
    echo "shadow_contract_file: $shadow_contract"
    echo "generation_report_json: $generation_report_json"
    echo "shadow_report_json: $shadow_report_json"
    echo "generation_counterexample_triage_json: $generation_counterexample_triage_json"
    echo "generation_counterexample_triage_txt: $generation_counterexample_triage_txt"
    echo "shadow_counterexample_triage_json: $shadow_counterexample_triage_json"
    echo "shadow_counterexample_triage_txt: $shadow_counterexample_triage_txt"
    echo "generation_parser_rejections_total: $generation_parser_rejections"
    echo "generation_counterexamples_count: $generation_counterexamples_count"
    echo "generation_counterexample_unique_shrunk_samples: $generation_counterexample_unique_shrunk_samples"
    echo "generation_counterexample_unique_failure_locations: $generation_counterexample_unique_failure_locations"
    echo "shadow_parser_rejections_total: $shadow_parser_rejections"
    echo "shadow_counterexamples_count: $shadow_counterexamples_count"
    echo "shadow_counterexamples_captured_total: $shadow_counterexamples_captured_total"
    echo "shadow_counterexample_unique_shrunk_samples: $shadow_counterexample_unique_shrunk_samples"
    echo "shadow_counterexample_unique_failure_locations: $shadow_counterexample_unique_failure_locations"
    echo "focused_initial_target_count: $initial_target_count"
    echo "focused_replay_target_count: $replay_target_count"
    echo "focused_initial_covered_reachable_rules: $initial_covered_reachable_rules"
    echo "focused_replay_covered_reachable_rules: $replay_covered_reachable_rules"
    echo "focused_initial_covered_reachable_branches: $initial_covered_reachable_branches"
    echo "focused_replay_covered_reachable_branches: $replay_covered_reachable_branches"
} | tee "$SUMMARY_TXT"

echo "✅ SV parser aggregate contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
