#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_parser_aggregate_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

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
    and ((.counterexamples // []) | type == "array")
    and (((.counterexamples // []) | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0 and has("counterexamples")
        then ((.counterexamples // []) | length) > 0
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
        and has("failure_line_excerpt")
        and has("failure_context_excerpt")
        and has("profile")
        and has("sample_index")
        and has("seed")
        and (.failure_position | type == "number")
        and (.failure_line | type == "number")
        and (.failure_column | type == "number")
        and (.failure_line_excerpt | type == "string")
        and (.failure_context_excerpt | type == "string")
        and (if has("primary_entry_rule") then (.primary_entry_rule | type == "string") else true end)
        and (if has("generation_entry_rule") then (.generation_entry_rule | type == "string") else true end)
        and (if has("entry_mode") then (.entry_mode | type == "string") else true end)
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
    and ((.counterexamples // []) | type == "array")
    and ((.counterexamples_captured_total // 0) | numbers) >= (((.counterexamples // []) | length))
    and (((.counterexamples // []) | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0 and has("counterexamples")
        then ((.counterexamples // []) | length) > 0
        else true
        end
    )
    and ((.profiles | length) >= 1)
    and all(.profiles[]; ((.counterexamples_captured // 0) | numbers) >= 0)
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
        and has("parser_error")
        and has("failure_position")
        and has("failure_line")
        and has("failure_column")
        and has("failure_line_excerpt")
        and has("failure_context_excerpt")
        and has("profile")
        and (.failure_position | type == "number")
        and (.failure_line | type == "number")
        and (.failure_column | type == "number")
        and (.failure_line_excerpt | type == "string")
        and (.failure_context_excerpt | type == "string")
        and (if has("primary_entry_rule") then (.primary_entry_rule | type == "string") else true end)
        and (if has("generation_entry_rule") then (.generation_entry_rule | type == "string") else true end)
        and (if has("entry_mode") then (.entry_mode | type == "string") else true end)
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
    and (
        (.target_drive_validation.helper_timeout_errors_total | numbers)
        <= (.observed.generation_errors_total | numbers)
    )
    and (
        (.target_drive_validation.helper_timeout_errors_total | numbers)
        <= (.target_drive_validation.alternate_entry_attempts_total | numbers)
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
replay_gap_target_triage_json="$WORK_DIR/systemverilog_replay_gap_target_triage.json"
replay_gap_target_triage_txt="$WORK_DIR/systemverilog_replay_gap_target_triage.txt"

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
        by_primary_entry_rule: (
            (.counterexamples // [])
            | sort_by(.primary_entry_rule // "<missing>")
            | group_by(.primary_entry_rule // "<missing>")
            | map({
                primary_entry_rule: (.[0].primary_entry_rule // "<missing>"),
                count: length
            })
        ),
        by_generation_entry_rule: (
            (.counterexamples // [])
            | sort_by(.generation_entry_rule // "<missing>")
            | group_by(.generation_entry_rule // "<missing>")
            | map({
                generation_entry_rule: (.[0].generation_entry_rule // "<missing>"),
                count: length
            })
        ),
        by_entry_mode: (
            (.counterexamples // [])
            | sort_by(.entry_mode // "<missing>")
            | group_by(.entry_mode // "<missing>")
            | map({
                entry_mode: (.[0].entry_mode // "<missing>"),
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
        by_failure_line_excerpt: (
            (.counterexamples // [])
            | sort_by(.failure_line_excerpt)
            | group_by(.failure_line_excerpt)
            | map({
                failure_line_excerpt: .[0].failure_line_excerpt,
                count: length
            })
        ),
        by_failure_context_excerpt: (
            (.counterexamples // [])
            | sort_by(.failure_context_excerpt)
            | group_by(.failure_context_excerpt)
            | map({
                failure_context_excerpt: .[0].failure_context_excerpt,
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
                primary_entry_rule: (.primary_entry_rule // "<missing>"),
                generation_entry_rule: (.generation_entry_rule // "<missing>"),
                entry_mode: (.entry_mode // "<missing>"),
                shrunk_sample,
                failure_line_excerpt,
                failure_context_excerpt,
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
    jq -r '.by_primary_entry_rule[]? | "primary_entry_rule_count[\(.primary_entry_rule | @json)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_generation_entry_rule[]? | "generation_entry_rule_count[\(.generation_entry_rule | @json)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_entry_mode[]? | "entry_mode_count[\(.entry_mode | @json)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_failure_line_excerpt[]? | "failure_line_excerpt_count[\(.failure_line_excerpt | @json)]: \(.count)"' "$generation_counterexample_triage_json"
    jq -r '.by_failure_context_excerpt[]? | "failure_context_excerpt_count[\(.failure_context_excerpt | @json)]: \(.count)"' "$generation_counterexample_triage_json"
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
        by_primary_entry_rule: (
            (.counterexamples // [])
            | sort_by(.primary_entry_rule // "<missing>")
            | group_by(.primary_entry_rule // "<missing>")
            | map({
                primary_entry_rule: (.[0].primary_entry_rule // "<missing>"),
                count: length
            })
        ),
        by_generation_entry_rule: (
            (.counterexamples // [])
            | sort_by(.generation_entry_rule // "<missing>")
            | group_by(.generation_entry_rule // "<missing>")
            | map({
                generation_entry_rule: (.[0].generation_entry_rule // "<missing>"),
                count: length
            })
        ),
        by_entry_mode: (
            (.counterexamples // [])
            | sort_by(.entry_mode // "<missing>")
            | group_by(.entry_mode // "<missing>")
            | map({
                entry_mode: (.[0].entry_mode // "<missing>"),
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
        by_failure_line_excerpt: (
            (.counterexamples // [])
            | sort_by(.failure_line_excerpt)
            | group_by(.failure_line_excerpt)
            | map({
                failure_line_excerpt: .[0].failure_line_excerpt,
                count: length
            })
        ),
        by_failure_context_excerpt: (
            (.counterexamples // [])
            | sort_by(.failure_context_excerpt)
            | group_by(.failure_context_excerpt)
            | map({
                failure_context_excerpt: .[0].failure_context_excerpt,
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
                primary_entry_rule: (.primary_entry_rule // "<missing>"),
                generation_entry_rule: (.generation_entry_rule // "<missing>"),
                entry_mode: (.entry_mode // "<missing>"),
                shrunk_sample,
                failure_line_excerpt,
                failure_context_excerpt,
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
    jq -r '.by_primary_entry_rule[]? | "primary_entry_rule_count[\(.primary_entry_rule | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_generation_entry_rule[]? | "generation_entry_rule_count[\(.generation_entry_rule | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_entry_mode[]? | "entry_mode_count[\(.entry_mode | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_failure_line_excerpt[]? | "failure_line_excerpt_count[\(.failure_line_excerpt | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
    jq -r '.by_failure_context_excerpt[]? | "failure_context_excerpt_count[\(.failure_context_excerpt | @json)]: \(.count)"' "$shadow_counterexample_triage_json"
} >"$shadow_counterexample_triage_txt"
require_nonempty_file "$shadow_counterexample_triage_txt"

jq '
    {
        grammar_name: .grammar_name,
        aggregate_surface: "replay_gap_targets",
        total_targets: ((.targets // []) | length),
        by_target_type: (
            (.targets // [])
            | sort_by(.target_type)
            | group_by(.target_type)
            | map({
                target_type: .[0].target_type,
                count: length
            })
        ),
        by_rule_name: (
            (.targets // [])
            | sort_by(.rule_name)
            | group_by(.rule_name)
            | map({
                rule_name: .[0].rule_name,
                count: length
            })
            | sort_by(-.count, .rule_name)
        ),
        by_reason: (
            (.targets // [])
            | sort_by(.reason)
            | group_by(.reason)
            | map({
                reason: .[0].reason,
                count: length
            })
            | sort_by(-.count, .reason)
        ),
        by_dependency: (
            (.targets // [])
            | map(.depends_on // [])
            | add
            | sort
            | group_by(.)
            | map({
                dependency: .[0],
                count: length
            })
            | sort_by(-.count, .dependency)
        ),
        highest_priority_targets: (
            (.targets // [])
            | sort_by(-.priority_score, .rule_name, .branch_index, .id)
            | .[:10]
            | map({
                id,
                target_type,
                rule_name,
                branch_index,
                reason,
                priority_score,
                depends_on
            })
        )
    }
' "$closed_loop_replay_gap_json" >"$replay_gap_target_triage_json"
require_nonempty_file "$replay_gap_target_triage_json"

{
    echo "SV Parser Replay Gap Target Triage"
    echo "source_gap_json: $closed_loop_replay_gap_json"
    jq -r '.by_target_type[]? | "target_type_count[\(.target_type)]: \(.count)"' "$replay_gap_target_triage_json"
    jq -r '.by_reason[]? | "reason_count[\(.reason)]: \(.count)"' "$replay_gap_target_triage_json"
    jq -r '(.by_rule_name[:10])[]? | "top_rule_count[\(.rule_name)]: \(.count)"' "$replay_gap_target_triage_json"
    jq -r '(.by_dependency[:10])[]? | "top_dependency_count[\(.dependency)]: \(.count)"' "$replay_gap_target_triage_json"
} >"$replay_gap_target_triage_txt"
require_nonempty_file "$replay_gap_target_triage_txt"

generation_parser_rejections="$(extract_json_number "$generation_report_json" '.observed.parser_rejections_total')"
generation_counterexamples_count="$(extract_json_number "$generation_report_json" '((.counterexamples // []) | length)')"
shadow_parser_rejections="$(extract_json_number "$shadow_report_json" '.observed.parser_rejections_total')"
shadow_counterexamples_count="$(extract_json_number "$shadow_report_json" '((.counterexamples // []) | length)')"
shadow_counterexamples_captured_total="$(extract_json_number "$shadow_report_json" '(.counterexamples_captured_total // 0)')"
generation_counterexample_unique_shrunk_samples="$(extract_json_number "$generation_counterexample_triage_json" '(.by_shrunk_sample | length)')"
generation_counterexample_unique_primary_entry_rules="$(extract_json_number "$generation_counterexample_triage_json" '(.by_primary_entry_rule | length)')"
generation_counterexample_unique_generation_entry_rules="$(extract_json_number "$generation_counterexample_triage_json" '(.by_generation_entry_rule | length)')"
generation_counterexample_unique_entry_modes="$(extract_json_number "$generation_counterexample_triage_json" '(.by_entry_mode | length)')"
generation_counterexample_unique_failure_locations="$(extract_json_number "$generation_counterexample_triage_json" '(.by_failure_location | length)')"
generation_counterexample_unique_failure_line_excerpts="$(extract_json_number "$generation_counterexample_triage_json" '(.by_failure_line_excerpt | length)')"
generation_counterexample_unique_failure_context_excerpts="$(extract_json_number "$generation_counterexample_triage_json" '(.by_failure_context_excerpt | length)')"
generation_counterexample_primary_stage="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].stage) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_stage_count="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_shrunk_sample="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].shrunk_sample) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_shrunk_sample_count="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_primary_entry_rule="$(jq -er 'if (.by_primary_entry_rule | length) > 0 then (.by_primary_entry_rule | sort_by(-.count, .primary_entry_rule) | .[0].primary_entry_rule) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_primary_entry_rule_count="$(jq -er 'if (.by_primary_entry_rule | length) > 0 then (.by_primary_entry_rule | sort_by(-.count, .primary_entry_rule) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_generation_entry_rule="$(jq -er 'if (.by_generation_entry_rule | length) > 0 then (.by_generation_entry_rule | sort_by(-.count, .generation_entry_rule) | .[0].generation_entry_rule) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_generation_entry_rule_count="$(jq -er 'if (.by_generation_entry_rule | length) > 0 then (.by_generation_entry_rule | sort_by(-.count, .generation_entry_rule) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_entry_mode="$(jq -er 'if (.by_entry_mode | length) > 0 then (.by_entry_mode | sort_by(-.count, .entry_mode) | .[0].entry_mode) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_entry_mode_count="$(jq -er 'if (.by_entry_mode | length) > 0 then (.by_entry_mode | sort_by(-.count, .entry_mode) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_parser_error="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].parser_error) else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_parser_error_count="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_location="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0] | "\(.failure_line):\(.failure_column)") else "<none>" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_location_count="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_line_excerpt_json="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].failure_line_excerpt | @json) else "\"<none>\"" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_line_excerpt_count="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_context_excerpt_json="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].failure_context_excerpt | @json) else "\"<none>\"" end' "$generation_counterexample_triage_json")"
generation_counterexample_primary_failure_context_excerpt_count="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].count) else 0 end' "$generation_counterexample_triage_json")"
shadow_counterexample_unique_shrunk_samples="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_shrunk_sample | length)')"
shadow_counterexample_unique_primary_entry_rules="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_primary_entry_rule | length)')"
shadow_counterexample_unique_generation_entry_rules="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_generation_entry_rule | length)')"
shadow_counterexample_unique_entry_modes="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_entry_mode | length)')"
shadow_counterexample_unique_failure_locations="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_failure_location | length)')"
shadow_counterexample_unique_failure_line_excerpts="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_failure_line_excerpt | length)')"
shadow_counterexample_unique_failure_context_excerpts="$(extract_json_number "$shadow_counterexample_triage_json" '(.by_failure_context_excerpt | length)')"
shadow_counterexample_primary_stage="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].stage) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_stage_count="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_shrunk_sample="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].shrunk_sample) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_shrunk_sample_count="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_primary_entry_rule="$(jq -er 'if (.by_primary_entry_rule | length) > 0 then (.by_primary_entry_rule | sort_by(-.count, .primary_entry_rule) | .[0].primary_entry_rule) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_primary_entry_rule_count="$(jq -er 'if (.by_primary_entry_rule | length) > 0 then (.by_primary_entry_rule | sort_by(-.count, .primary_entry_rule) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_generation_entry_rule="$(jq -er 'if (.by_generation_entry_rule | length) > 0 then (.by_generation_entry_rule | sort_by(-.count, .generation_entry_rule) | .[0].generation_entry_rule) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_generation_entry_rule_count="$(jq -er 'if (.by_generation_entry_rule | length) > 0 then (.by_generation_entry_rule | sort_by(-.count, .generation_entry_rule) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_entry_mode="$(jq -er 'if (.by_entry_mode | length) > 0 then (.by_entry_mode | sort_by(-.count, .entry_mode) | .[0].entry_mode) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_entry_mode_count="$(jq -er 'if (.by_entry_mode | length) > 0 then (.by_entry_mode | sort_by(-.count, .entry_mode) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_parser_error="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].parser_error) else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_parser_error_count="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_location="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0] | "\(.failure_line):\(.failure_column)") else "<none>" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_location_count="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_line_excerpt_json="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].failure_line_excerpt | @json) else "\"<none>\"" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_line_excerpt_count="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_context_excerpt_json="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].failure_context_excerpt | @json) else "\"<none>\"" end' "$shadow_counterexample_triage_json")"
shadow_counterexample_primary_failure_context_excerpt_count="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].count) else 0 end' "$shadow_counterexample_triage_json")"
replay_gap_target_unique_rules="$(extract_json_number "$replay_gap_target_triage_json" '(.by_rule_name | length)')"
replay_gap_target_unique_reasons="$(extract_json_number "$replay_gap_target_triage_json" '(.by_reason | length)')"
replay_gap_target_unique_dependencies="$(extract_json_number "$replay_gap_target_triage_json" '(.by_dependency | length)')"
replay_gap_target_primary_target_type="$(jq -er 'if (.by_target_type | length) > 0 then (.by_target_type | sort_by(-.count, .target_type) | .[0].target_type) else "<none>" end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_target_type_count="$(jq -er 'if (.by_target_type | length) > 0 then (.by_target_type | sort_by(-.count, .target_type) | .[0].count) else 0 end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_reason="$(jq -er 'if (.by_reason | length) > 0 then (.by_reason | sort_by(-.count, .reason) | .[0].reason) else "<none>" end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_reason_count="$(jq -er 'if (.by_reason | length) > 0 then (.by_reason | sort_by(-.count, .reason) | .[0].count) else 0 end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_rule="$(jq -er 'if (.by_rule_name | length) > 0 then (.by_rule_name | sort_by(-.count, .rule_name) | .[0].rule_name) else "<none>" end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_rule_count="$(jq -er 'if (.by_rule_name | length) > 0 then (.by_rule_name | sort_by(-.count, .rule_name) | .[0].count) else 0 end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_dependency="$(jq -er 'if (.by_dependency | length) > 0 then (.by_dependency | sort_by(-.count, .dependency) | .[0].dependency) else "<none>" end' "$replay_gap_target_triage_json")"
replay_gap_target_primary_dependency_count="$(jq -er 'if (.by_dependency | length) > 0 then (.by_dependency | sort_by(-.count, .dependency) | .[0].count) else 0 end' "$replay_gap_target_triage_json")"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Parser Aggregate Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
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
    echo "replay_gap_target_triage_json: $replay_gap_target_triage_json"
    echo "replay_gap_target_triage_txt: $replay_gap_target_triage_txt"
    echo "source_gap_json: $closed_loop_replay_gap_json"
    echo "generation_parser_rejections_total: $generation_parser_rejections"
    echo "generation_counterexamples_count: $generation_counterexamples_count"
    echo "generation_counterexample_unique_shrunk_samples: $generation_counterexample_unique_shrunk_samples"
    echo "generation_counterexample_unique_primary_entry_rules: $generation_counterexample_unique_primary_entry_rules"
    echo "generation_counterexample_unique_generation_entry_rules: $generation_counterexample_unique_generation_entry_rules"
    echo "generation_counterexample_unique_entry_modes: $generation_counterexample_unique_entry_modes"
    echo "generation_counterexample_primary_stage: $generation_counterexample_primary_stage"
    echo "generation_counterexample_primary_stage_count: $generation_counterexample_primary_stage_count"
    echo "generation_counterexample_primary_shrunk_sample: $generation_counterexample_primary_shrunk_sample"
    echo "generation_counterexample_primary_shrunk_sample_count: $generation_counterexample_primary_shrunk_sample_count"
    echo "generation_counterexample_primary_primary_entry_rule: $generation_counterexample_primary_primary_entry_rule"
    echo "generation_counterexample_primary_primary_entry_rule_count: $generation_counterexample_primary_primary_entry_rule_count"
    echo "generation_counterexample_primary_generation_entry_rule: $generation_counterexample_primary_generation_entry_rule"
    echo "generation_counterexample_primary_generation_entry_rule_count: $generation_counterexample_primary_generation_entry_rule_count"
    echo "generation_counterexample_primary_entry_mode: $generation_counterexample_primary_entry_mode"
    echo "generation_counterexample_primary_entry_mode_count: $generation_counterexample_primary_entry_mode_count"
    echo "generation_counterexample_primary_parser_error: $generation_counterexample_primary_parser_error"
    echo "generation_counterexample_primary_parser_error_count: $generation_counterexample_primary_parser_error_count"
    echo "generation_counterexample_primary_failure_location: $generation_counterexample_primary_failure_location"
    echo "generation_counterexample_primary_failure_location_count: $generation_counterexample_primary_failure_location_count"
    echo "generation_counterexample_primary_failure_line_excerpt_json: $generation_counterexample_primary_failure_line_excerpt_json"
    echo "generation_counterexample_primary_failure_line_excerpt_count: $generation_counterexample_primary_failure_line_excerpt_count"
    echo "generation_counterexample_primary_failure_context_excerpt_json: $generation_counterexample_primary_failure_context_excerpt_json"
    echo "generation_counterexample_primary_failure_context_excerpt_count: $generation_counterexample_primary_failure_context_excerpt_count"
    echo "generation_counterexample_unique_failure_locations: $generation_counterexample_unique_failure_locations"
    echo "generation_counterexample_unique_failure_line_excerpts: $generation_counterexample_unique_failure_line_excerpts"
    echo "generation_counterexample_unique_failure_context_excerpts: $generation_counterexample_unique_failure_context_excerpts"
    echo "shadow_parser_rejections_total: $shadow_parser_rejections"
    echo "shadow_counterexamples_count: $shadow_counterexamples_count"
    echo "shadow_counterexamples_captured_total: $shadow_counterexamples_captured_total"
    echo "shadow_counterexample_unique_shrunk_samples: $shadow_counterexample_unique_shrunk_samples"
    echo "shadow_counterexample_unique_primary_entry_rules: $shadow_counterexample_unique_primary_entry_rules"
    echo "shadow_counterexample_unique_generation_entry_rules: $shadow_counterexample_unique_generation_entry_rules"
    echo "shadow_counterexample_unique_entry_modes: $shadow_counterexample_unique_entry_modes"
    echo "shadow_counterexample_primary_stage: $shadow_counterexample_primary_stage"
    echo "shadow_counterexample_primary_stage_count: $shadow_counterexample_primary_stage_count"
    echo "shadow_counterexample_primary_shrunk_sample: $shadow_counterexample_primary_shrunk_sample"
    echo "shadow_counterexample_primary_shrunk_sample_count: $shadow_counterexample_primary_shrunk_sample_count"
    echo "shadow_counterexample_primary_primary_entry_rule: $shadow_counterexample_primary_primary_entry_rule"
    echo "shadow_counterexample_primary_primary_entry_rule_count: $shadow_counterexample_primary_primary_entry_rule_count"
    echo "shadow_counterexample_primary_generation_entry_rule: $shadow_counterexample_primary_generation_entry_rule"
    echo "shadow_counterexample_primary_generation_entry_rule_count: $shadow_counterexample_primary_generation_entry_rule_count"
    echo "shadow_counterexample_primary_entry_mode: $shadow_counterexample_primary_entry_mode"
    echo "shadow_counterexample_primary_entry_mode_count: $shadow_counterexample_primary_entry_mode_count"
    echo "shadow_counterexample_primary_parser_error: $shadow_counterexample_primary_parser_error"
    echo "shadow_counterexample_primary_parser_error_count: $shadow_counterexample_primary_parser_error_count"
    echo "shadow_counterexample_primary_failure_location: $shadow_counterexample_primary_failure_location"
    echo "shadow_counterexample_primary_failure_location_count: $shadow_counterexample_primary_failure_location_count"
    echo "shadow_counterexample_primary_failure_line_excerpt_json: $shadow_counterexample_primary_failure_line_excerpt_json"
    echo "shadow_counterexample_primary_failure_line_excerpt_count: $shadow_counterexample_primary_failure_line_excerpt_count"
    echo "shadow_counterexample_primary_failure_context_excerpt_json: $shadow_counterexample_primary_failure_context_excerpt_json"
    echo "shadow_counterexample_primary_failure_context_excerpt_count: $shadow_counterexample_primary_failure_context_excerpt_count"
    echo "shadow_counterexample_unique_failure_locations: $shadow_counterexample_unique_failure_locations"
    echo "shadow_counterexample_unique_failure_line_excerpts: $shadow_counterexample_unique_failure_line_excerpts"
    echo "shadow_counterexample_unique_failure_context_excerpts: $shadow_counterexample_unique_failure_context_excerpts"
    echo "replay_gap_target_unique_rules: $replay_gap_target_unique_rules"
    echo "replay_gap_target_unique_reasons: $replay_gap_target_unique_reasons"
    echo "replay_gap_target_unique_dependencies: $replay_gap_target_unique_dependencies"
    echo "replay_gap_target_primary_target_type: $replay_gap_target_primary_target_type"
    echo "replay_gap_target_primary_target_type_count: $replay_gap_target_primary_target_type_count"
    echo "replay_gap_target_primary_reason: $replay_gap_target_primary_reason"
    echo "replay_gap_target_primary_reason_count: $replay_gap_target_primary_reason_count"
    echo "replay_gap_target_primary_rule: $replay_gap_target_primary_rule"
    echo "replay_gap_target_primary_rule_count: $replay_gap_target_primary_rule_count"
    echo "replay_gap_target_primary_dependency: $replay_gap_target_primary_dependency"
    echo "replay_gap_target_primary_dependency_count: $replay_gap_target_primary_dependency_count"
    echo "focused_initial_target_count: $initial_target_count"
    echo "focused_replay_target_count: $replay_target_count"
    echo "focused_initial_covered_reachable_rules: $initial_covered_reachable_rules"
    echo "focused_replay_covered_reachable_rules: $replay_covered_reachable_rules"
    echo "focused_initial_covered_reachable_branches: $initial_covered_reachable_branches"
    echo "focused_replay_covered_reachable_branches: $replay_covered_reachable_branches"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_parser_aggregate_contract_gate" \
    --argjson version 1 \
    --arg state_dir "$STATE_DIR" \
    --arg generated_at_utc "$generated_at_utc" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg existing_sv_stimuli_quality_state_dir "${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-<unset>}" \
    --arg base_contract_file "$BASE_CONTRACT_FILE" \
    --arg generation_contract_file "$generation_contract" \
    --arg shadow_contract_file "$shadow_contract" \
    --arg generation_state_dir "$generation_state_dir" \
    --arg shadow_state_dir "$shadow_state_dir" \
    --arg generation_report_json "$generation_report_json" \
    --arg shadow_report_json "$shadow_report_json" \
    --arg generation_counterexample_triage_json "$generation_counterexample_triage_json" \
    --arg generation_counterexample_triage_txt "$generation_counterexample_triage_txt" \
    --arg shadow_counterexample_triage_json "$shadow_counterexample_triage_json" \
    --arg shadow_counterexample_triage_txt "$shadow_counterexample_triage_txt" \
    --arg replay_gap_target_triage_json "$replay_gap_target_triage_json" \
    --arg replay_gap_target_triage_txt "$replay_gap_target_triage_txt" \
    --arg source_gap_json "$closed_loop_replay_gap_json" \
    --argjson generation_parser_rejections_total "$generation_parser_rejections" \
    --argjson generation_counterexamples_count "$generation_counterexamples_count" \
    --argjson generation_counterexample_unique_shrunk_samples "$generation_counterexample_unique_shrunk_samples" \
    --argjson generation_counterexample_unique_primary_entry_rules "$generation_counterexample_unique_primary_entry_rules" \
    --argjson generation_counterexample_unique_generation_entry_rules "$generation_counterexample_unique_generation_entry_rules" \
    --argjson generation_counterexample_unique_entry_modes "$generation_counterexample_unique_entry_modes" \
    --arg generation_counterexample_primary_stage "$generation_counterexample_primary_stage" \
    --argjson generation_counterexample_primary_stage_count "$generation_counterexample_primary_stage_count" \
    --arg generation_counterexample_primary_shrunk_sample "$generation_counterexample_primary_shrunk_sample" \
    --argjson generation_counterexample_primary_shrunk_sample_count "$generation_counterexample_primary_shrunk_sample_count" \
    --arg generation_counterexample_primary_primary_entry_rule "$generation_counterexample_primary_primary_entry_rule" \
    --argjson generation_counterexample_primary_primary_entry_rule_count "$generation_counterexample_primary_primary_entry_rule_count" \
    --arg generation_counterexample_primary_generation_entry_rule "$generation_counterexample_primary_generation_entry_rule" \
    --argjson generation_counterexample_primary_generation_entry_rule_count "$generation_counterexample_primary_generation_entry_rule_count" \
    --arg generation_counterexample_primary_entry_mode "$generation_counterexample_primary_entry_mode" \
    --argjson generation_counterexample_primary_entry_mode_count "$generation_counterexample_primary_entry_mode_count" \
    --arg generation_counterexample_primary_parser_error "$generation_counterexample_primary_parser_error" \
    --argjson generation_counterexample_primary_parser_error_count "$generation_counterexample_primary_parser_error_count" \
    --arg generation_counterexample_primary_failure_location "$generation_counterexample_primary_failure_location" \
    --argjson generation_counterexample_primary_failure_location_count "$generation_counterexample_primary_failure_location_count" \
    --arg generation_counterexample_primary_failure_line_excerpt_json "$generation_counterexample_primary_failure_line_excerpt_json" \
    --argjson generation_counterexample_primary_failure_line_excerpt_count "$generation_counterexample_primary_failure_line_excerpt_count" \
    --arg generation_counterexample_primary_failure_context_excerpt_json "$generation_counterexample_primary_failure_context_excerpt_json" \
    --argjson generation_counterexample_primary_failure_context_excerpt_count "$generation_counterexample_primary_failure_context_excerpt_count" \
    --argjson generation_counterexample_unique_failure_locations "$generation_counterexample_unique_failure_locations" \
    --argjson generation_counterexample_unique_failure_line_excerpts "$generation_counterexample_unique_failure_line_excerpts" \
    --argjson generation_counterexample_unique_failure_context_excerpts "$generation_counterexample_unique_failure_context_excerpts" \
    --argjson shadow_parser_rejections_total "$shadow_parser_rejections" \
    --argjson shadow_counterexamples_count "$shadow_counterexamples_count" \
    --argjson shadow_counterexamples_captured_total "$shadow_counterexamples_captured_total" \
    --argjson shadow_counterexample_unique_shrunk_samples "$shadow_counterexample_unique_shrunk_samples" \
    --argjson shadow_counterexample_unique_primary_entry_rules "$shadow_counterexample_unique_primary_entry_rules" \
    --argjson shadow_counterexample_unique_generation_entry_rules "$shadow_counterexample_unique_generation_entry_rules" \
    --argjson shadow_counterexample_unique_entry_modes "$shadow_counterexample_unique_entry_modes" \
    --arg shadow_counterexample_primary_stage "$shadow_counterexample_primary_stage" \
    --argjson shadow_counterexample_primary_stage_count "$shadow_counterexample_primary_stage_count" \
    --arg shadow_counterexample_primary_shrunk_sample "$shadow_counterexample_primary_shrunk_sample" \
    --argjson shadow_counterexample_primary_shrunk_sample_count "$shadow_counterexample_primary_shrunk_sample_count" \
    --arg shadow_counterexample_primary_primary_entry_rule "$shadow_counterexample_primary_primary_entry_rule" \
    --argjson shadow_counterexample_primary_primary_entry_rule_count "$shadow_counterexample_primary_primary_entry_rule_count" \
    --arg shadow_counterexample_primary_generation_entry_rule "$shadow_counterexample_primary_generation_entry_rule" \
    --argjson shadow_counterexample_primary_generation_entry_rule_count "$shadow_counterexample_primary_generation_entry_rule_count" \
    --arg shadow_counterexample_primary_entry_mode "$shadow_counterexample_primary_entry_mode" \
    --argjson shadow_counterexample_primary_entry_mode_count "$shadow_counterexample_primary_entry_mode_count" \
    --arg shadow_counterexample_primary_parser_error "$shadow_counterexample_primary_parser_error" \
    --argjson shadow_counterexample_primary_parser_error_count "$shadow_counterexample_primary_parser_error_count" \
    --arg shadow_counterexample_primary_failure_location "$shadow_counterexample_primary_failure_location" \
    --argjson shadow_counterexample_primary_failure_location_count "$shadow_counterexample_primary_failure_location_count" \
    --arg shadow_counterexample_primary_failure_line_excerpt_json "$shadow_counterexample_primary_failure_line_excerpt_json" \
    --argjson shadow_counterexample_primary_failure_line_excerpt_count "$shadow_counterexample_primary_failure_line_excerpt_count" \
    --arg shadow_counterexample_primary_failure_context_excerpt_json "$shadow_counterexample_primary_failure_context_excerpt_json" \
    --argjson shadow_counterexample_primary_failure_context_excerpt_count "$shadow_counterexample_primary_failure_context_excerpt_count" \
    --argjson shadow_counterexample_unique_failure_locations "$shadow_counterexample_unique_failure_locations" \
    --argjson shadow_counterexample_unique_failure_line_excerpts "$shadow_counterexample_unique_failure_line_excerpts" \
    --argjson shadow_counterexample_unique_failure_context_excerpts "$shadow_counterexample_unique_failure_context_excerpts" \
    --argjson replay_gap_target_unique_rules "$replay_gap_target_unique_rules" \
    --argjson replay_gap_target_unique_reasons "$replay_gap_target_unique_reasons" \
    --argjson replay_gap_target_unique_dependencies "$replay_gap_target_unique_dependencies" \
    --arg replay_gap_target_primary_target_type "$replay_gap_target_primary_target_type" \
    --argjson replay_gap_target_primary_target_type_count "$replay_gap_target_primary_target_type_count" \
    --arg replay_gap_target_primary_reason "$replay_gap_target_primary_reason" \
    --argjson replay_gap_target_primary_reason_count "$replay_gap_target_primary_reason_count" \
    --arg replay_gap_target_primary_rule "$replay_gap_target_primary_rule" \
    --argjson replay_gap_target_primary_rule_count "$replay_gap_target_primary_rule_count" \
    --arg replay_gap_target_primary_dependency "$replay_gap_target_primary_dependency" \
    --argjson replay_gap_target_primary_dependency_count "$replay_gap_target_primary_dependency_count" \
    --argjson focused_initial_target_count "$initial_target_count" \
    --argjson focused_replay_target_count "$replay_target_count" \
    --argjson focused_initial_covered_reachable_rules "$initial_covered_reachable_rules" \
    --argjson focused_replay_covered_reachable_rules "$replay_covered_reachable_rules" \
    --argjson focused_initial_covered_reachable_branches "$initial_covered_reachable_branches" \
    --argjson focused_replay_covered_reachable_branches "$replay_covered_reachable_branches" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      proof_surfaces: {
        generation_state_dir: $generation_state_dir,
        shadow_state_dir: $shadow_state_dir,
        generation_report_json: $generation_report_json,
        shadow_report_json: $shadow_report_json,
        generation_counterexample_triage_json: $generation_counterexample_triage_json,
        generation_counterexample_triage_txt: $generation_counterexample_triage_txt,
        shadow_counterexample_triage_json: $shadow_counterexample_triage_json,
        shadow_counterexample_triage_txt: $shadow_counterexample_triage_txt,
        replay_gap_target_triage_json: $replay_gap_target_triage_json,
        replay_gap_target_triage_txt: $replay_gap_target_triage_txt,
        source_gap_json: $source_gap_json
      },
      metrics: {
        existing_sv_stimuli_quality_state_dir: $existing_sv_stimuli_quality_state_dir,
        base_contract_file: $base_contract_file,
        generation_contract_file: $generation_contract_file,
        shadow_contract_file: $shadow_contract_file,
        generation_parser_rejections_total: $generation_parser_rejections_total,
        generation_counterexamples_count: $generation_counterexamples_count,
        generation_counterexample_unique_shrunk_samples: $generation_counterexample_unique_shrunk_samples,
        generation_counterexample_unique_primary_entry_rules: $generation_counterexample_unique_primary_entry_rules,
        generation_counterexample_unique_generation_entry_rules: $generation_counterexample_unique_generation_entry_rules,
        generation_counterexample_unique_entry_modes: $generation_counterexample_unique_entry_modes,
        generation_counterexample_primary_stage: $generation_counterexample_primary_stage,
        generation_counterexample_primary_stage_count: $generation_counterexample_primary_stage_count,
        generation_counterexample_primary_shrunk_sample: $generation_counterexample_primary_shrunk_sample,
        generation_counterexample_primary_shrunk_sample_count: $generation_counterexample_primary_shrunk_sample_count,
        generation_counterexample_primary_primary_entry_rule: $generation_counterexample_primary_primary_entry_rule,
        generation_counterexample_primary_primary_entry_rule_count: $generation_counterexample_primary_primary_entry_rule_count,
        generation_counterexample_primary_generation_entry_rule: $generation_counterexample_primary_generation_entry_rule,
        generation_counterexample_primary_generation_entry_rule_count: $generation_counterexample_primary_generation_entry_rule_count,
        generation_counterexample_primary_entry_mode: $generation_counterexample_primary_entry_mode,
        generation_counterexample_primary_entry_mode_count: $generation_counterexample_primary_entry_mode_count,
        generation_counterexample_primary_parser_error: $generation_counterexample_primary_parser_error,
        generation_counterexample_primary_parser_error_count: $generation_counterexample_primary_parser_error_count,
        generation_counterexample_primary_failure_location: $generation_counterexample_primary_failure_location,
        generation_counterexample_primary_failure_location_count: $generation_counterexample_primary_failure_location_count,
        generation_counterexample_primary_failure_line_excerpt_json: $generation_counterexample_primary_failure_line_excerpt_json,
        generation_counterexample_primary_failure_line_excerpt_count: $generation_counterexample_primary_failure_line_excerpt_count,
        generation_counterexample_primary_failure_context_excerpt_json: $generation_counterexample_primary_failure_context_excerpt_json,
        generation_counterexample_primary_failure_context_excerpt_count: $generation_counterexample_primary_failure_context_excerpt_count,
        generation_counterexample_unique_failure_locations: $generation_counterexample_unique_failure_locations,
        generation_counterexample_unique_failure_line_excerpts: $generation_counterexample_unique_failure_line_excerpts,
        generation_counterexample_unique_failure_context_excerpts: $generation_counterexample_unique_failure_context_excerpts,
        shadow_parser_rejections_total: $shadow_parser_rejections_total,
        shadow_counterexamples_count: $shadow_counterexamples_count,
        shadow_counterexamples_captured_total: $shadow_counterexamples_captured_total,
        shadow_counterexample_unique_shrunk_samples: $shadow_counterexample_unique_shrunk_samples,
        shadow_counterexample_unique_primary_entry_rules: $shadow_counterexample_unique_primary_entry_rules,
        shadow_counterexample_unique_generation_entry_rules: $shadow_counterexample_unique_generation_entry_rules,
        shadow_counterexample_unique_entry_modes: $shadow_counterexample_unique_entry_modes,
        shadow_counterexample_primary_stage: $shadow_counterexample_primary_stage,
        shadow_counterexample_primary_stage_count: $shadow_counterexample_primary_stage_count,
        shadow_counterexample_primary_shrunk_sample: $shadow_counterexample_primary_shrunk_sample,
        shadow_counterexample_primary_shrunk_sample_count: $shadow_counterexample_primary_shrunk_sample_count,
        shadow_counterexample_primary_primary_entry_rule: $shadow_counterexample_primary_primary_entry_rule,
        shadow_counterexample_primary_primary_entry_rule_count: $shadow_counterexample_primary_primary_entry_rule_count,
        shadow_counterexample_primary_generation_entry_rule: $shadow_counterexample_primary_generation_entry_rule,
        shadow_counterexample_primary_generation_entry_rule_count: $shadow_counterexample_primary_generation_entry_rule_count,
        shadow_counterexample_primary_entry_mode: $shadow_counterexample_primary_entry_mode,
        shadow_counterexample_primary_entry_mode_count: $shadow_counterexample_primary_entry_mode_count,
        shadow_counterexample_primary_parser_error: $shadow_counterexample_primary_parser_error,
        shadow_counterexample_primary_parser_error_count: $shadow_counterexample_primary_parser_error_count,
        shadow_counterexample_primary_failure_location: $shadow_counterexample_primary_failure_location,
        shadow_counterexample_primary_failure_location_count: $shadow_counterexample_primary_failure_location_count,
        shadow_counterexample_primary_failure_line_excerpt_json: $shadow_counterexample_primary_failure_line_excerpt_json,
        shadow_counterexample_primary_failure_line_excerpt_count: $shadow_counterexample_primary_failure_line_excerpt_count,
        shadow_counterexample_primary_failure_context_excerpt_json: $shadow_counterexample_primary_failure_context_excerpt_json,
        shadow_counterexample_primary_failure_context_excerpt_count: $shadow_counterexample_primary_failure_context_excerpt_count,
        shadow_counterexample_unique_failure_locations: $shadow_counterexample_unique_failure_locations,
        shadow_counterexample_unique_failure_line_excerpts: $shadow_counterexample_unique_failure_line_excerpts,
        shadow_counterexample_unique_failure_context_excerpts: $shadow_counterexample_unique_failure_context_excerpts,
        replay_gap_target_unique_rules: $replay_gap_target_unique_rules,
        replay_gap_target_unique_reasons: $replay_gap_target_unique_reasons,
        replay_gap_target_unique_dependencies: $replay_gap_target_unique_dependencies,
        replay_gap_target_primary_target_type: $replay_gap_target_primary_target_type,
        replay_gap_target_primary_target_type_count: $replay_gap_target_primary_target_type_count,
        replay_gap_target_primary_reason: $replay_gap_target_primary_reason,
        replay_gap_target_primary_reason_count: $replay_gap_target_primary_reason_count,
        replay_gap_target_primary_rule: $replay_gap_target_primary_rule,
        replay_gap_target_primary_rule_count: $replay_gap_target_primary_rule_count,
        replay_gap_target_primary_dependency: $replay_gap_target_primary_dependency,
        replay_gap_target_primary_dependency_count: $replay_gap_target_primary_dependency_count,
        focused_initial_target_count: $focused_initial_target_count,
        focused_replay_target_count: $focused_replay_target_count,
        focused_initial_covered_reachable_rules: $focused_initial_covered_reachable_rules,
        focused_replay_covered_reachable_rules: $focused_replay_covered_reachable_rules,
        focused_initial_covered_reachable_branches: $focused_initial_covered_reachable_branches,
        focused_replay_covered_reachable_branches: $focused_replay_covered_reachable_branches
      }
    }' >"$SUMMARY_JSON"

echo "✅ SV parser aggregate contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
