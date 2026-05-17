#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_aggregate_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
EXISTING_QUALITY_STATE_DIR="${PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR:-}"

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

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | numbers" "$path"
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

canonicalize_json() {
    local source="$1"
    local target="$2"
    jq -S . "$source" >"$target"
}

assert_same_text() {
    local left="$1"
    local right="$2"
    local context="$3"
    if ! cmp -s "$left" "$right"; then
        echo "error: deterministic replay mismatch for $context" >&2
        diff -u "$left" "$right" | head -n 80 >&2 || true
        exit 1
    fi
}

assert_same_json() {
    local left="$1"
    local right="$2"
    local context="$3"
    local left_norm="${left}.norm.json"
    local right_norm="${right}.norm.json"
    canonicalize_json "$left" "$left_norm"
    canonicalize_json "$right" "$right_norm"
    assert_same_text "$left_norm" "$right_norm" "$context"
}

require_tool jq
if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    require_file "$QUALITY_GATE_SCRIPT"
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

quality_state_dir="${EXISTING_QUALITY_STATE_DIR:-$WORK_DIR/quality_state}"

if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    run_logged "preprocessor_quality_probe" \
        env \
            PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$quality_state_dir" \
            PGEN_SV_PREPROCESSOR_DIFF_MODE=0 \
            "$QUALITY_GATE_SCRIPT"
fi

parseability_report_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_report.json"
counterexample_triage_json="$WORK_DIR/systemverilog_preprocessor_parseability_counterexample_triage.json"
counterexample_triage_txt="$WORK_DIR/systemverilog_preprocessor_parseability_counterexample_triage.txt"
samples_stage0_a_txt="$quality_state_dir/work/systemverilog_preprocessor_samples_stage0_a.txt"
samples_stage0_b_txt="$quality_state_dir/work/systemverilog_preprocessor_samples_stage0_b.txt"
coverage_stage0_a_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage0_a.json"
coverage_stage0_b_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage0_b.json"
gap_stage3_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage3.json"
gap_stage0_a_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage0_a.json"
gap_stage0_b_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage0_b.json"
gap_stage1_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage1.json"
coverage_stage1_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage1.json"
coverage_stage3_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage3.json"
parseability_stage0_a_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_stage0_a.json"
parseability_stage0_b_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_stage0_b.json"
parseability_stage2_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_stage2.json"
samples_stage4_a_txt="$quality_state_dir/work/systemverilog_preprocessor_samples_stage4_fuzz_a.txt"
samples_stage4_b_txt="$quality_state_dir/work/systemverilog_preprocessor_samples_stage4_fuzz_b.txt"
coverage_stage4_a_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage4_fuzz_a.json"
coverage_stage4_b_json="$quality_state_dir/work/systemverilog_preprocessor_coverage_stage4_fuzz_b.json"
gap_stage4_a_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage4_fuzz_a.json"
gap_stage4_b_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage4_fuzz_b.json"
fuzz_replay_a_json="$quality_state_dir/work/systemverilog_preprocessor_fuzz_replay_a.json"
fuzz_replay_b_json="$quality_state_dir/work/systemverilog_preprocessor_fuzz_replay_b.json"

require_nonempty_file "$parseability_report_json"
require_nonempty_file "$samples_stage0_a_txt"
require_nonempty_file "$samples_stage0_b_txt"
require_nonempty_file "$coverage_stage0_a_json"
require_nonempty_file "$coverage_stage0_b_json"
require_nonempty_file "$gap_stage0_a_json"
require_nonempty_file "$gap_stage0_b_json"
require_nonempty_file "$gap_stage1_json"
require_nonempty_file "$gap_stage3_json"
require_nonempty_file "$coverage_stage1_json"
require_nonempty_file "$coverage_stage3_json"
require_nonempty_file "$parseability_stage0_a_json"
require_nonempty_file "$parseability_stage0_b_json"
require_nonempty_file "$parseability_stage2_json"
require_nonempty_file "$samples_stage4_a_txt"
require_nonempty_file "$samples_stage4_b_txt"
require_nonempty_file "$coverage_stage4_a_json"
require_nonempty_file "$coverage_stage4_b_json"
require_nonempty_file "$gap_stage4_a_json"
require_nonempty_file "$gap_stage4_b_json"
require_nonempty_file "$fuzz_replay_a_json"
require_nonempty_file "$fuzz_replay_b_json"

assert_same_text "$samples_stage0_a_txt" "$samples_stage0_b_txt" "preprocessor stage0 sample corpus"
assert_same_json "$coverage_stage0_a_json" "$coverage_stage0_b_json" "preprocessor stage0 coverage metrics"
assert_same_json "$gap_stage0_a_json" "$gap_stage0_b_json" "preprocessor stage0 gap report"
assert_same_json "$parseability_stage0_a_json" "$parseability_stage0_b_json" "preprocessor stage0 parseability report"
assert_same_text "$samples_stage4_a_txt" "$samples_stage4_b_txt" "preprocessor stage4 fuzz sample corpus"
assert_same_json "$coverage_stage4_a_json" "$coverage_stage4_b_json" "preprocessor stage4 fuzz coverage metrics"
assert_same_json "$gap_stage4_a_json" "$gap_stage4_b_json" "preprocessor stage4 fuzz gap report"
assert_same_json "$fuzz_replay_a_json" "$fuzz_replay_b_json" "preprocessor stage4 fuzz replay metadata"

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and .effective_mode == "enabled"
    and (.counterexamples | type == "array")
    and ((.counterexamples | length) <= 20)
    and (
        if (.summary.parser_rejections // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and (.stages.stage0_baseline.summary | type == "object")
    and (.stages.stage1_gap_priority.summary | type == "object")
    and (.stages.stage2_target_drive.summary | type == "object")
    and (.stages.stage3_recompute_gap.summary | type == "object")
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
        and (.failure_position | type == "number")
        and (.failure_line | type == "number")
        and (.failure_column | type == "number")
        and (.failure_line_excerpt | type == "string")
        and (.failure_context_excerpt | type == "string")
    )
' "$parseability_report_json" >/dev/null; then
    echo "error: preprocessor aggregate parseability report contract failed: $parseability_report_json" >&2
    cat "$parseability_report_json" >&2
    exit 1
fi

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and (.targets | type == "array")
    and ((.targets | length) == 0)
    and (.summary.covered_reachable_rules == .summary.reachable_rules)
    and (.summary.covered_reachable_branches == .summary.reachable_branches)
' "$gap_stage3_json" >/dev/null; then
    echo "error: preprocessor final gap contract failed: $gap_stage3_json" >&2
    cat "$gap_stage3_json" >&2
    exit 1
fi

assert_json "$parseability_report_json" '
    (.summary.attempts | numbers)
        == (
            (.stages.stage0_baseline.summary.attempts | numbers)
            + (.stages.stage1_gap_priority.summary.attempts | numbers)
            + (.stages.stage2_target_drive.summary.attempts | numbers)
            + (.stages.stage3_recompute_gap.summary.attempts | numbers)
        )
    and (.summary.accepted | numbers)
        == (
            (.stages.stage0_baseline.summary.accepted | numbers)
            + (.stages.stage1_gap_priority.summary.accepted | numbers)
            + (.stages.stage2_target_drive.summary.accepted | numbers)
            + (.stages.stage3_recompute_gap.summary.accepted | numbers)
        )
    and (.summary.rejected | numbers)
        == (
            (.stages.stage0_baseline.summary.rejected | numbers)
            + (.stages.stage1_gap_priority.summary.rejected | numbers)
            + (.stages.stage2_target_drive.summary.rejected | numbers)
            + (.stages.stage3_recompute_gap.summary.rejected | numbers)
        )
    and (.summary.parser_rejections | numbers)
        == (
            (.stages.stage0_baseline.summary.parser_rejections | numbers)
            + (.stages.stage1_gap_priority.summary.parser_rejections | numbers)
            + (.stages.stage2_target_drive.summary.parser_rejections | numbers)
            + (.stages.stage3_recompute_gap.summary.parser_rejections | numbers)
        )
    and (.summary.generation_errors | numbers)
        == (
            (.stages.stage0_baseline.summary.generation_errors | numbers)
            + (.stages.stage1_gap_priority.summary.generation_errors | numbers)
            + (.stages.stage2_target_drive.summary.generation_errors | numbers)
            + (.stages.stage3_recompute_gap.summary.generation_errors | numbers)
        )
    and (.summary.empty_generations | numbers)
        == (
            (.stages.stage0_baseline.summary.empty_generations | numbers)
            + (.stages.stage1_gap_priority.summary.empty_generations | numbers)
            + (.stages.stage2_target_drive.summary.empty_generations | numbers)
            + (.stages.stage3_recompute_gap.summary.empty_generations | numbers)
        )
    and (.target_drive_validation.primary_entry_attempts_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.primary_entry_attempts | numbers)
    and (.target_drive_validation.primary_entry_accepted_outputs_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.primary_entry_accepted_outputs | numbers)
    and (.target_drive_validation.primary_entry_rejected_outputs_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.primary_entry_rejected_outputs | numbers)
    and (.target_drive_validation.alternate_entry_attempts_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.alternate_entry_attempts | numbers)
    and (.target_drive_validation.alternate_entry_accepted_outputs_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.alternate_entry_accepted_outputs | numbers)
    and (.target_drive_validation.alternate_entry_rejected_outputs_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.alternate_entry_rejected_outputs | numbers)
    and (.target_drive_validation.target_timeout_errors_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.target_timeout_errors | numbers)
    and (.target_drive_validation.helper_timeout_errors_total | numbers)
        == (.stages.stage2_target_drive.target_drive_validation.helper_timeout_errors | numbers)
' "preprocessor aggregate parseability totals are internally inconsistent"

assert_json "$fuzz_replay_a_json" '
    (.rounds | numbers) >= 1
    and ((.cases | length) == (.rounds | numbers))
    and ((.accepted_cases | numbers) + (.rejected_cases | numbers) == (.rounds | numbers))
    and ((.minimized_cases | numbers) <= (.rounds | numbers))
    and ((.shrunk_counterexamples | numbers) <= (.parseability_counterexamples | numbers))
' "preprocessor fuzz replay aggregate invariants failed"

stage0_target_count="$(extract_json_number "$gap_stage0_a_json" '((.targets // []) | length)')"
stage1_target_count="$(extract_json_number "$gap_stage1_json" '((.targets // []) | length)')"
stage3_target_count="$(extract_json_number "$gap_stage3_json" '((.targets // []) | length)')"
stage4_target_count="$(extract_json_number "$gap_stage4_a_json" '((.targets // []) | length)')"
stage0_covered_reachable_rules="$(extract_json_number "$gap_stage0_a_json" '.summary.covered_reachable_rules')"
stage1_covered_reachable_rules="$(extract_json_number "$gap_stage1_json" '.summary.covered_reachable_rules')"
stage3_covered_reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_rules')"
stage4_covered_reachable_rules="$(extract_json_number "$gap_stage4_a_json" '.summary.covered_reachable_rules')"
stage0_reachable_rules="$(extract_json_number "$gap_stage0_a_json" '.summary.reachable_rules')"
stage1_reachable_rules="$(extract_json_number "$gap_stage1_json" '.summary.reachable_rules')"
stage3_reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.reachable_rules')"
stage4_reachable_rules="$(extract_json_number "$gap_stage4_a_json" '.summary.reachable_rules')"
stage0_covered_reachable_branches="$(extract_json_number "$gap_stage0_a_json" '.summary.covered_reachable_branches')"
stage1_covered_reachable_branches="$(extract_json_number "$gap_stage1_json" '.summary.covered_reachable_branches')"
stage3_covered_reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_branches')"
stage4_covered_reachable_branches="$(extract_json_number "$gap_stage4_a_json" '.summary.covered_reachable_branches')"
stage0_reachable_branches="$(extract_json_number "$gap_stage0_a_json" '.summary.reachable_branches')"
stage1_reachable_branches="$(extract_json_number "$gap_stage1_json" '.summary.reachable_branches')"
stage3_reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.reachable_branches')"
stage4_reachable_branches="$(extract_json_number "$gap_stage4_a_json" '.summary.reachable_branches')"
# Static universe (stage-invariant by definition: pure grammar+entry property).
stage0_total_rules="$(extract_json_number "$gap_stage0_a_json" '.summary.total_rules')"
stage1_total_rules="$(extract_json_number "$gap_stage1_json" '.summary.total_rules')"
stage3_total_rules="$(extract_json_number "$gap_stage3_json" '.summary.total_rules')"
stage4_total_rules="$(extract_json_number "$gap_stage4_a_json" '.summary.total_rules')"
stage0_total_branches="$(extract_json_number "$gap_stage0_a_json" '.summary.total_branches')"
stage1_total_branches="$(extract_json_number "$gap_stage1_json" '.summary.total_branches')"
stage3_total_branches="$(extract_json_number "$gap_stage3_json" '.summary.total_branches')"
stage4_total_branches="$(extract_json_number "$gap_stage4_a_json" '.summary.total_branches')"

if (( stage1_target_count > stage0_target_count )); then
    echo "error: preprocessor gap-priority target debt increased: stage0=$stage0_target_count stage1=$stage1_target_count" >&2
    exit 1
fi

if (( stage3_target_count > stage1_target_count )); then
    echo "error: preprocessor recompute-gap target debt increased: stage1=$stage1_target_count stage3=$stage3_target_count" >&2
    exit 1
fi

if (( stage4_target_count > stage3_target_count )); then
    echo "error: preprocessor fuzz replay target debt increased: stage3=$stage3_target_count stage4=$stage4_target_count" >&2
    exit 1
fi

if (( stage1_covered_reachable_rules < stage0_covered_reachable_rules || stage3_covered_reachable_rules < stage1_covered_reachable_rules || stage4_covered_reachable_rules < stage3_covered_reachable_rules )); then
    echo "error: preprocessor reachable-rule coverage regressed across stages: stage0=$stage0_covered_reachable_rules stage1=$stage1_covered_reachable_rules stage3=$stage3_covered_reachable_rules stage4=$stage4_covered_reachable_rules" >&2
    exit 1
fi

if (( stage1_covered_reachable_branches < stage0_covered_reachable_branches || stage3_covered_reachable_branches < stage1_covered_reachable_branches || stage4_covered_reachable_branches < stage3_covered_reachable_branches )); then
    echo "error: preprocessor reachable-branch coverage regressed across stages: stage0=$stage0_covered_reachable_branches stage1=$stage1_covered_reachable_branches stage3=$stage3_covered_reachable_branches stage4=$stage4_covered_reachable_branches" >&2
    exit 1
fi

# Corrected 2026-05-17 (PGEN-SV-EXH-PROOF-0004, leaf SV-EXH-PROOF.2.2).
# The prior assertions required summary.reachable_rules / summary.reachable_branches
# to be byte-IDENTICAL across stage0/1/3/4 ("universe drifted"). That is
# mis-specified: in stimuli_generator.rs the gap report skips branches whose
# `deficit == 0` (already covered to threshold), so summary.reachable_* is a
# reachable-but-NOT-yet-covered *burn-down debt* count, NOT a static universe.
# It legitimately DECREASES as the closed loop covers more across stages
# (PGEN-POST-SV-AUDIT-0002's Cat-A macro_formals factoring added macro_default_atom
# branches that stage0 leaves uncovered (reachable_branches=10) and stage1 then
# covers (covered_branches 37->47, reachable_branches->0) -- desirable burn-down,
# wrongly flagged as drift). The genuine "static universe must not drift" intent
# is enforced correctly below on the truly stage-invariant total_rules /
# total_branches; the genuine no-regression guarantee on the debt metric is a
# monotone non-increasing (burn-down) check, matching the existing stage debt /
# coverage invariants above. Strengthened (true universe pinned) + corrected
# (debt non-increasing) -- not weakened; a real regression (debt GROWING across
# stages) is still caught.
if (( stage0_total_rules != stage1_total_rules || stage1_total_rules != stage3_total_rules || stage3_total_rules != stage4_total_rules || stage0_total_branches != stage1_total_branches || stage1_total_branches != stage3_total_branches || stage3_total_branches != stage4_total_branches )); then
    echo "error: preprocessor static rule/branch universe drifted across stages: total_rules stage0=$stage0_total_rules stage1=$stage1_total_rules stage3=$stage3_total_rules stage4=$stage4_total_rules ; total_branches stage0=$stage0_total_branches stage1=$stage1_total_branches stage3=$stage3_total_branches stage4=$stage4_total_branches" >&2
    exit 1
fi

if (( stage1_reachable_rules > stage0_reachable_rules || stage3_reachable_rules > stage1_reachable_rules || stage4_reachable_rules > stage3_reachable_rules )); then
    echo "error: preprocessor reachable-rule debt increased across stages (burn-down regression): stage0=$stage0_reachable_rules stage1=$stage1_reachable_rules stage3=$stage3_reachable_rules stage4=$stage4_reachable_rules" >&2
    exit 1
fi

if (( stage1_reachable_branches > stage0_reachable_branches || stage3_reachable_branches > stage1_reachable_branches || stage4_reachable_branches > stage3_reachable_branches )); then
    echo "error: preprocessor reachable-branch debt increased across stages (burn-down regression): stage0=$stage0_reachable_branches stage1=$stage1_reachable_branches stage3=$stage3_reachable_branches stage4=$stage4_reachable_branches" >&2
    exit 1
fi

jq '
    {
        grammar_name: .grammar_name,
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
                shrunk_sample,
                failure_line_excerpt,
                failure_context_excerpt,
                sample_preview: (.sample[:80])
            })
        )
    }
' "$parseability_report_json" >"$counterexample_triage_json"
require_nonempty_file "$counterexample_triage_json"

{
    echo "SV Preprocessor Counterexample Triage"
    echo "source_report: $parseability_report_json"
    jq -r '.by_stage[]? | "stage_count[\(.stage)]: \(.count)"' "$counterexample_triage_json"
    jq -r '.by_shrunk_sample[]? | "shrunk_sample_count[\(.shrunk_sample | @json)]: \(.count)"' "$counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$counterexample_triage_json"
    jq -r '.by_failure_line_excerpt[]? | "failure_line_excerpt_count[\(.failure_line_excerpt | @json)]: \(.count)"' "$counterexample_triage_json"
    jq -r '.by_failure_context_excerpt[]? | "failure_context_excerpt_count[\(.failure_context_excerpt | @json)]: \(.count)"' "$counterexample_triage_json"
} >"$counterexample_triage_txt"
require_nonempty_file "$counterexample_triage_txt"

parseability_attempts_total="$(extract_json_number "$parseability_report_json" '.summary.attempts')"
parseability_accepted_total="$(extract_json_number "$parseability_report_json" '.summary.accepted')"
parseability_rejected_total="$(extract_json_number "$parseability_report_json" '.summary.rejected')"
parseability_parser_rejections_total="$(extract_json_number "$parseability_report_json" '.summary.parser_rejections')"
parseability_counterexamples_captured_total="$(extract_json_number "$parseability_report_json" '((.counterexamples // []) | length)')"
final_targets="$(extract_json_number "$gap_stage3_json" '(.targets | length)')"
covered_reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_rules')"
reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.reachable_rules')"
covered_reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_branches')"
reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.reachable_branches')"
fuzz_replay_accepted_cases="$(extract_json_number "$fuzz_replay_a_json" '.accepted_cases')"
fuzz_replay_rejected_cases="$(extract_json_number "$fuzz_replay_a_json" '.rejected_cases')"
fuzz_replay_parseability_counterexamples="$(extract_json_number "$fuzz_replay_a_json" '.parseability_counterexamples')"
counterexample_unique_shrunk_samples="$(extract_json_number "$counterexample_triage_json" '(.by_shrunk_sample | length)')"
counterexample_unique_failure_locations="$(extract_json_number "$counterexample_triage_json" '(.by_failure_location | length)')"
counterexample_unique_failure_line_excerpts="$(extract_json_number "$counterexample_triage_json" '(.by_failure_line_excerpt | length)')"
counterexample_unique_failure_context_excerpts="$(extract_json_number "$counterexample_triage_json" '(.by_failure_context_excerpt | length)')"
counterexample_primary_stage="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].stage) else "<none>" end' "$counterexample_triage_json")"
counterexample_primary_stage_count="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].count) else 0 end' "$counterexample_triage_json")"
counterexample_primary_shrunk_sample="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].shrunk_sample) else "<none>" end' "$counterexample_triage_json")"
counterexample_primary_shrunk_sample_count="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].count) else 0 end' "$counterexample_triage_json")"
counterexample_primary_parser_error="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].parser_error) else "<none>" end' "$counterexample_triage_json")"
counterexample_primary_parser_error_count="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].count) else 0 end' "$counterexample_triage_json")"
counterexample_primary_failure_location="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0] | "\(.failure_line):\(.failure_column)") else "<none>" end' "$counterexample_triage_json")"
counterexample_primary_failure_location_count="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0].count) else 0 end' "$counterexample_triage_json")"
counterexample_primary_failure_line_excerpt_json="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].failure_line_excerpt | @json) else "\"<none>\"" end' "$counterexample_triage_json")"
counterexample_primary_failure_line_excerpt_count="$(jq -er 'if (.by_failure_line_excerpt | length) > 0 then (.by_failure_line_excerpt | sort_by(-.count, .failure_line_excerpt) | .[0].count) else 0 end' "$counterexample_triage_json")"
counterexample_primary_failure_context_excerpt_json="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].failure_context_excerpt | @json) else "\"<none>\"" end' "$counterexample_triage_json")"
counterexample_primary_failure_context_excerpt_count="$(jq -er 'if (.by_failure_context_excerpt | length) > 0 then (.by_failure_context_excerpt | sort_by(-.count, .failure_context_excerpt) | .[0].count) else 0 end' "$counterexample_triage_json")"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Preprocessor Aggregate Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "existing_quality_state_dir: ${EXISTING_QUALITY_STATE_DIR:-<unset>}"
    echo "quality_state_dir: $quality_state_dir"
    echo "parseability_report_json: $parseability_report_json"
    echo "counterexample_triage_json: $counterexample_triage_json"
    echo "counterexample_triage_txt: $counterexample_triage_txt"
    echo "gap_stage3_json: $gap_stage3_json"
    echo "parseability_attempts_total: $parseability_attempts_total"
    echo "parseability_accepted_total: $parseability_accepted_total"
    echo "parseability_rejected_total: $parseability_rejected_total"
    echo "parseability_parser_rejections_total: $parseability_parser_rejections_total"
    echo "parseability_counterexamples_captured_total: $parseability_counterexamples_captured_total"
    echo "counterexample_unique_shrunk_samples: $counterexample_unique_shrunk_samples"
    echo "counterexample_primary_stage: $counterexample_primary_stage"
    echo "counterexample_primary_stage_count: $counterexample_primary_stage_count"
    echo "counterexample_primary_shrunk_sample: $counterexample_primary_shrunk_sample"
    echo "counterexample_primary_shrunk_sample_count: $counterexample_primary_shrunk_sample_count"
    echo "counterexample_primary_parser_error: $counterexample_primary_parser_error"
    echo "counterexample_primary_parser_error_count: $counterexample_primary_parser_error_count"
    echo "counterexample_primary_failure_location: $counterexample_primary_failure_location"
    echo "counterexample_primary_failure_location_count: $counterexample_primary_failure_location_count"
    echo "counterexample_primary_failure_line_excerpt_json: $counterexample_primary_failure_line_excerpt_json"
    echo "counterexample_primary_failure_line_excerpt_count: $counterexample_primary_failure_line_excerpt_count"
    echo "counterexample_primary_failure_context_excerpt_json: $counterexample_primary_failure_context_excerpt_json"
    echo "counterexample_primary_failure_context_excerpt_count: $counterexample_primary_failure_context_excerpt_count"
    echo "counterexample_unique_failure_locations: $counterexample_unique_failure_locations"
    echo "counterexample_unique_failure_line_excerpts: $counterexample_unique_failure_line_excerpts"
    echo "counterexample_unique_failure_context_excerpts: $counterexample_unique_failure_context_excerpts"
    echo "stage0_target_count: $stage0_target_count"
    echo "stage1_target_count: $stage1_target_count"
    echo "final_targets: $final_targets"
    echo "stage4_target_count: $stage4_target_count"
    echo "stage0_covered_reachable_rules: $stage0_covered_reachable_rules/$stage0_reachable_rules"
    echo "stage1_covered_reachable_rules: $stage1_covered_reachable_rules/$stage1_reachable_rules"
    echo "covered_reachable_rules: $covered_reachable_rules/$reachable_rules"
    echo "stage4_covered_reachable_rules: $stage4_covered_reachable_rules/$stage4_reachable_rules"
    echo "stage0_covered_reachable_branches: $stage0_covered_reachable_branches/$stage0_reachable_branches"
    echo "stage1_covered_reachable_branches: $stage1_covered_reachable_branches/$stage1_reachable_branches"
    echo "covered_reachable_branches: $covered_reachable_branches/$reachable_branches"
    echo "stage4_covered_reachable_branches: $stage4_covered_reachable_branches/$stage4_reachable_branches"
    echo "fuzz_replay_accepted_cases: $fuzz_replay_accepted_cases"
    echo "fuzz_replay_rejected_cases: $fuzz_replay_rejected_cases"
    echo "fuzz_replay_parseability_counterexamples: $fuzz_replay_parseability_counterexamples"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_preprocessor_aggregate_contract_gate" \
    --argjson version 1 \
    --arg state_dir "$STATE_DIR" \
    --arg generated_at_utc "$generated_at_utc" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg existing_quality_state_dir "${EXISTING_QUALITY_STATE_DIR:-<unset>}" \
    --arg quality_state_dir "$quality_state_dir" \
    --arg parseability_report_json "$parseability_report_json" \
    --arg counterexample_triage_json "$counterexample_triage_json" \
    --arg counterexample_triage_txt "$counterexample_triage_txt" \
    --arg gap_stage3_json "$gap_stage3_json" \
    --argjson parseability_attempts_total "$parseability_attempts_total" \
    --argjson parseability_accepted_total "$parseability_accepted_total" \
    --argjson parseability_rejected_total "$parseability_rejected_total" \
    --argjson parseability_parser_rejections_total "$parseability_parser_rejections_total" \
    --argjson parseability_counterexamples_captured_total "$parseability_counterexamples_captured_total" \
    --argjson counterexample_unique_shrunk_samples "$counterexample_unique_shrunk_samples" \
    --arg counterexample_primary_stage "$counterexample_primary_stage" \
    --argjson counterexample_primary_stage_count "$counterexample_primary_stage_count" \
    --arg counterexample_primary_shrunk_sample "$counterexample_primary_shrunk_sample" \
    --argjson counterexample_primary_shrunk_sample_count "$counterexample_primary_shrunk_sample_count" \
    --arg counterexample_primary_parser_error "$counterexample_primary_parser_error" \
    --argjson counterexample_primary_parser_error_count "$counterexample_primary_parser_error_count" \
    --arg counterexample_primary_failure_location "$counterexample_primary_failure_location" \
    --argjson counterexample_primary_failure_location_count "$counterexample_primary_failure_location_count" \
    --arg counterexample_primary_failure_line_excerpt_json "$counterexample_primary_failure_line_excerpt_json" \
    --argjson counterexample_primary_failure_line_excerpt_count "$counterexample_primary_failure_line_excerpt_count" \
    --arg counterexample_primary_failure_context_excerpt_json "$counterexample_primary_failure_context_excerpt_json" \
    --argjson counterexample_primary_failure_context_excerpt_count "$counterexample_primary_failure_context_excerpt_count" \
    --argjson counterexample_unique_failure_locations "$counterexample_unique_failure_locations" \
    --argjson counterexample_unique_failure_line_excerpts "$counterexample_unique_failure_line_excerpts" \
    --argjson counterexample_unique_failure_context_excerpts "$counterexample_unique_failure_context_excerpts" \
    --argjson stage0_target_count "$stage0_target_count" \
    --argjson stage1_target_count "$stage1_target_count" \
    --argjson final_targets "$final_targets" \
    --argjson stage4_target_count "$stage4_target_count" \
    --arg stage0_covered_reachable_rules "$(printf '%s/%s' "$stage0_covered_reachable_rules" "$stage0_reachable_rules")" \
    --arg stage1_covered_reachable_rules "$(printf '%s/%s' "$stage1_covered_reachable_rules" "$stage1_reachable_rules")" \
    --arg covered_reachable_rules "$(printf '%s/%s' "$covered_reachable_rules" "$reachable_rules")" \
    --arg stage4_covered_reachable_rules "$(printf '%s/%s' "$stage4_covered_reachable_rules" "$stage4_reachable_rules")" \
    --arg stage0_covered_reachable_branches "$(printf '%s/%s' "$stage0_covered_reachable_branches" "$stage0_reachable_branches")" \
    --arg stage1_covered_reachable_branches "$(printf '%s/%s' "$stage1_covered_reachable_branches" "$stage1_reachable_branches")" \
    --arg covered_reachable_branches "$(printf '%s/%s' "$covered_reachable_branches" "$reachable_branches")" \
    --arg stage4_covered_reachable_branches "$(printf '%s/%s' "$stage4_covered_reachable_branches" "$stage4_reachable_branches")" \
    --argjson fuzz_replay_accepted_cases "$fuzz_replay_accepted_cases" \
    --argjson fuzz_replay_rejected_cases "$fuzz_replay_rejected_cases" \
    --argjson fuzz_replay_parseability_counterexamples "$fuzz_replay_parseability_counterexamples" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      proof_surfaces: {
        quality_state_dir: $quality_state_dir,
        parseability_report_json: $parseability_report_json,
        counterexample_triage_json: $counterexample_triage_json,
        counterexample_triage_txt: $counterexample_triage_txt,
        gap_stage3_json: $gap_stage3_json
      },
      metrics: {
        existing_quality_state_dir: $existing_quality_state_dir,
        parseability_attempts_total: $parseability_attempts_total,
        parseability_accepted_total: $parseability_accepted_total,
        parseability_rejected_total: $parseability_rejected_total,
        parseability_parser_rejections_total: $parseability_parser_rejections_total,
        parseability_counterexamples_captured_total: $parseability_counterexamples_captured_total,
        counterexample_unique_shrunk_samples: $counterexample_unique_shrunk_samples,
        counterexample_primary_stage: $counterexample_primary_stage,
        counterexample_primary_stage_count: $counterexample_primary_stage_count,
        counterexample_primary_shrunk_sample: $counterexample_primary_shrunk_sample,
        counterexample_primary_shrunk_sample_count: $counterexample_primary_shrunk_sample_count,
        counterexample_primary_parser_error: $counterexample_primary_parser_error,
        counterexample_primary_parser_error_count: $counterexample_primary_parser_error_count,
        counterexample_primary_failure_location: $counterexample_primary_failure_location,
        counterexample_primary_failure_location_count: $counterexample_primary_failure_location_count,
        counterexample_primary_failure_line_excerpt_json: $counterexample_primary_failure_line_excerpt_json,
        counterexample_primary_failure_line_excerpt_count: $counterexample_primary_failure_line_excerpt_count,
        counterexample_primary_failure_context_excerpt_json: $counterexample_primary_failure_context_excerpt_json,
        counterexample_primary_failure_context_excerpt_count: $counterexample_primary_failure_context_excerpt_count,
        counterexample_unique_failure_locations: $counterexample_unique_failure_locations,
        counterexample_unique_failure_line_excerpts: $counterexample_unique_failure_line_excerpts,
        counterexample_unique_failure_context_excerpts: $counterexample_unique_failure_context_excerpts,
        stage0_target_count: $stage0_target_count,
        stage1_target_count: $stage1_target_count,
        final_targets: $final_targets,
        stage4_target_count: $stage4_target_count,
        stage0_covered_reachable_rules: $stage0_covered_reachable_rules,
        stage1_covered_reachable_rules: $stage1_covered_reachable_rules,
        covered_reachable_rules: $covered_reachable_rules,
        stage4_covered_reachable_rules: $stage4_covered_reachable_rules,
        stage0_covered_reachable_branches: $stage0_covered_reachable_branches,
        stage1_covered_reachable_branches: $stage1_covered_reachable_branches,
        covered_reachable_branches: $covered_reachable_branches,
        stage4_covered_reachable_branches: $stage4_covered_reachable_branches,
        fuzz_replay_accepted_cases: $fuzz_replay_accepted_cases,
        fuzz_replay_rejected_cases: $fuzz_replay_rejected_cases,
        fuzz_replay_parseability_counterexamples: $fuzz_replay_parseability_counterexamples
      }
    }' >"$SUMMARY_JSON"

echo "✅ SV preprocessor aggregate contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
