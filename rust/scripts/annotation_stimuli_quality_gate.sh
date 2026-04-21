#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GENERATED_DIR="$ROOT_DIR/generated"

STATE_DIR="${PGEN_ANNOTATION_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/annotation_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SAMPLE_COUNT="${PGEN_ANNOTATION_STIMULI_QUALITY_COUNT:-24}"
GAP_THRESHOLD="${PGEN_ANNOTATION_STIMULI_QUALITY_GAP_THRESHOLD:-1}"
TARGET_MAX_ATTEMPTS="${PGEN_ANNOTATION_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS:-6000}"
FILTER="${PGEN_ANNOTATION_STIMULI_QUALITY_FILTER:-all}"

RETURN_SEED_BASE="${PGEN_ANNOTATION_STIMULI_QUALITY_RETURN_SEED:-8101}"
SEMANTIC_SEED_BASE="${PGEN_ANNOTATION_STIMULI_QUALITY_SEMANTIC_SEED:-8201}"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_ANNOTATION_STIMULI_QUALITY_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$GAP_THRESHOLD" =~ ^[0-9]+$ ]] || [[ "$GAP_THRESHOLD" -lt 1 ]]; then
    echo "error: PGEN_ANNOTATION_STIMULI_QUALITY_GAP_THRESHOLD must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_ANNOTATION_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
case "$FILTER" in
    all|return|semantic) ;;
    *)
        echo "error: PGEN_ANNOTATION_STIMULI_QUALITY_FILTER must be one of: all, return, semantic" >&2
        exit 2
        ;;
esac

mkdir -p "$LOG_DIR" "$WORK_DIR"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
RETURN_JSON="$GENERATED_DIR/return_annotation.json"
SEMANTIC_JSON="$GENERATED_DIR/semantic_annotation.json"

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
        echo "    fail (${log_file})" >&2
        tail -n 60 "$log_file" >&2 || true
        exit 1
    fi
}

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
        tail -n 60 "$log_file" >&2 || true
        exit 1
    fi
}

extract_json_u64() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | numbers" "$path"
}

extract_json_str() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | strings" "$path"
}

parseability_summary_field_u64() {
    local path="$1"
    local field="$2"
    jq -er ".summary.${field} | numbers" "$path"
}

parseability_target_drive_field_u64() {
    local path="$1"
    local field="$2"
    jq -er "(.target_drive_validation.${field} // 0) | numbers" "$path"
}

parseability_acceptance_rate_percent() {
    local path="$1"
    local attempts accepted
    attempts="$(parseability_summary_field_u64 "$path" "attempts")"
    accepted="$(parseability_summary_field_u64 "$path" "accepted")"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$accepted" "$attempts"
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

parse_target_summary() {
    local log_path="$1"
    local line
    line="$(grep -E "Target-driven generation: resolved [0-9]+/[0-9]+ targets in [0-9]+ attempts" "$log_path" | tail -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: unable to locate target-driven summary in '$log_path'" >&2
        exit 1
    fi
    if [[ "$line" =~ resolved[[:space:]]+([0-9]+)/([0-9]+)[[:space:]]+targets[[:space:]]+in[[:space:]]+([0-9]+)[[:space:]]+attempts ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]} ${BASH_REMATCH[3]}"
        return
    fi
    echo "error: failed to parse target-driven summary line '$line'" >&2
    exit 1
}

closed_loop_for_grammar() {
    local label="$1"
    local grammar_name="$2"
    local grammar_json="$3"
    local seed_base="$4"

    local coverage0="$WORK_DIR/${label}_coverage_stage0.json"
    local coverage1="$WORK_DIR/${label}_coverage_stage1.json"
    local coverage2="$WORK_DIR/${label}_coverage_stage2.json"
    local coverage3="$WORK_DIR/${label}_coverage_stage3.json"

    local gap0="$WORK_DIR/${label}_gap_stage0.json"
    local gap1="$WORK_DIR/${label}_gap_stage1.json"
    local gap3="$WORK_DIR/${label}_gap_stage3.json"

    local samples0="$WORK_DIR/${label}_samples_stage0.txt"
    local samples1="$WORK_DIR/${label}_samples_stage1.txt"
    local samples2="$WORK_DIR/${label}_samples_stage2.txt"
    local samples3="$WORK_DIR/${label}_samples_stage3.txt"
    local stage0_parseability_json="$WORK_DIR/${label}_parseability_stage0.json"
    local stage1_parseability_json="$WORK_DIR/${label}_parseability_stage1.json"
    local stage2_parseability_json="$WORK_DIR/${label}_parseability_stage2.json"
    local stage3_parseability_json="$WORK_DIR/${label}_parseability_stage3.json"
    local parseability_report_json="$WORK_DIR/${label}_parseability_report.json"
    local stage0_parseability_attempts=0
    local stage0_parseability_accepted=0
    local stage0_parseability_rejected=0
    local stage0_parseability_parser_rejections=0
    local stage0_parseability_generation_errors=0
    local stage0_parseability_empty_generations=0
    local stage1_parseability_attempts=0
    local stage1_parseability_accepted=0
    local stage1_parseability_rejected=0
    local stage1_parseability_parser_rejections=0
    local stage1_parseability_generation_errors=0
    local stage1_parseability_empty_generations=0
    local stage2_parseability_attempts=0
    local stage2_parseability_accepted=0
    local stage2_parseability_rejected=0
    local stage2_parseability_parser_rejections=0
    local stage2_parseability_generation_errors=0
    local stage2_parseability_empty_generations=0
    local stage2_target_drive_alternate_entry_attempts=0
    local stage2_target_drive_alternate_entry_accepted_outputs=0
    local stage2_target_drive_alternate_entry_rejected_outputs=0
    local stage2_target_drive_target_timeout_errors=0
    local stage2_target_drive_helper_timeout_errors=0
    local stage3_parseability_attempts=0
    local stage3_parseability_accepted=0
    local stage3_parseability_rejected=0
    local stage3_parseability_parser_rejections=0
    local stage3_parseability_generation_errors=0
    local stage3_parseability_empty_generations=0
    local parseability_attempts_total=0
    local parseability_accepted_total=0
    local parseability_rejected_total=0
    local parseability_parser_rejections_total=0
    local parseability_generation_errors_total=0
    local parseability_empty_generations_total=0
    local parseability_acceptance_rate_total="0.00"

    run_logged "${label}_stage0_baseline" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$seed_base" \
        --validate-parseability \
        --parseability-report-json "$stage0_parseability_json" \
        --gap-report-threshold "$GAP_THRESHOLD" \
        --output "$samples0" \
        --coverage-output "$coverage0" \
        --gap-report-json "$gap0"

    require_nonempty_file "$samples0"
    require_nonempty_file "$coverage0"
    require_nonempty_file "$gap0"
    require_nonempty_file "$stage0_parseability_json"

    assert_json "$coverage0" ".grammar_name == \"$grammar_name\"" "coverage stage0 grammar_name mismatch"
    assert_json "$coverage0" ".total_rules > 0" "coverage stage0 must report total_rules > 0"
    assert_json "$coverage0" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage0 attempts consistency failed"
    assert_json "$coverage0" ".sample_successes >= $SAMPLE_COUNT" "coverage stage0 sample_successes below requested count"
    assert_json "$stage0_parseability_json" ".grammar_name == \"$grammar_name\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage0 parseability report contract mismatch"

    assert_json "$gap0" ".grammar_name == \"$grammar_name\"" "gap stage0 grammar_name mismatch"
    assert_json "$gap0" ".summary.required_successes_per_target == $GAP_THRESHOLD" "gap stage0 threshold mismatch"
    assert_json "$gap0" "all(.targets[]?; .reachable == true)" "gap stage0 contains non-reachable target"
    assert_json "$gap0" ".summary.total_rules >= (.summary.reachable_rules + .summary.unreachable_rules)" "gap stage0 rule summary invariants failed"
    assert_json "$gap0" ".summary.total_branches >= (.summary.reachable_branches + .summary.unreachable_branches)" "gap stage0 branch summary invariants failed"

    local attempts0 successes0 covered_rules0 covered_branches0 initial_targets
    attempts0="$(extract_json_u64 "$coverage0" ".sample_attempts")"
    successes0="$(extract_json_u64 "$coverage0" ".sample_successes")"
    covered_rules0="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage0")"
    covered_branches0="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage0")"
    initial_targets="$(jq -er '.targets | length | numbers' "$gap0")"
    stage0_parseability_attempts="$(parseability_summary_field_u64 "$stage0_parseability_json" "attempts")"
    stage0_parseability_accepted="$(parseability_summary_field_u64 "$stage0_parseability_json" "accepted")"
    stage0_parseability_rejected="$(parseability_summary_field_u64 "$stage0_parseability_json" "rejected")"
    stage0_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage0_parseability_json" "parser_rejections")"
    stage0_parseability_generation_errors="$(parseability_summary_field_u64 "$stage0_parseability_json" "generation_errors")"
    stage0_parseability_empty_generations="$(parseability_summary_field_u64 "$stage0_parseability_json" "empty_generations")"

    run_logged "${label}_stage1_gap_priority" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$((seed_base + 1))" \
        --validate-parseability \
        --parseability-report-json "$stage1_parseability_json" \
        --coverage-input "$coverage0" \
        --gap-priority-report-input "$gap0" \
        --output "$samples1" \
        --coverage-output "$coverage1" \
        --gap-report-json "$gap1" \
        --gap-report-threshold "$GAP_THRESHOLD"

    require_nonempty_file "$samples1"
    require_nonempty_file "$coverage1"
    require_nonempty_file "$gap1"
    require_nonempty_file "$stage1_parseability_json"

    assert_json "$coverage1" ".grammar_name == \"$grammar_name\"" "coverage stage1 grammar_name mismatch"
    assert_json "$coverage1" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage1 attempts consistency failed"
    assert_json "$gap1" ".grammar_name == \"$grammar_name\"" "gap stage1 grammar_name mismatch"
    assert_json "$stage1_parseability_json" ".grammar_name == \"$grammar_name\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage1 parseability report contract mismatch"

    local attempts1 successes1 covered_rules1 covered_branches1
    attempts1="$(extract_json_u64 "$coverage1" ".sample_attempts")"
    successes1="$(extract_json_u64 "$coverage1" ".sample_successes")"
    covered_rules1="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage1")"
    covered_branches1="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage1")"
    stage1_parseability_attempts="$(parseability_summary_field_u64 "$stage1_parseability_json" "attempts")"
    stage1_parseability_accepted="$(parseability_summary_field_u64 "$stage1_parseability_json" "accepted")"
    stage1_parseability_rejected="$(parseability_summary_field_u64 "$stage1_parseability_json" "rejected")"
    stage1_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage1_parseability_json" "parser_rejections")"
    stage1_parseability_generation_errors="$(parseability_summary_field_u64 "$stage1_parseability_json" "generation_errors")"
    stage1_parseability_empty_generations="$(parseability_summary_field_u64 "$stage1_parseability_json" "empty_generations")"

    if (( attempts1 <= attempts0 )); then
        echo "error: ${label} stage1 sample_attempts did not increase ($attempts0 -> $attempts1)" >&2
        exit 1
    fi
    if (( successes1 <= successes0 )); then
        echo "error: ${label} stage1 sample_successes did not increase ($successes0 -> $successes1)" >&2
        exit 1
    fi
    if (( covered_rules1 < covered_rules0 )); then
        echo "error: ${label} stage1 covered_rules regressed ($covered_rules0 -> $covered_rules1)" >&2
        exit 1
    fi
    if (( covered_branches1 < covered_branches0 )); then
        echo "error: ${label} stage1 covered_branches regressed ($covered_branches0 -> $covered_branches1)" >&2
        exit 1
    fi

    run_logged "${label}_stage2_target_drive" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --seed "$((seed_base + 2))" \
        --validate-parseability \
        --parseability-report-json "$stage2_parseability_json" \
        --coverage-input "$coverage1" \
        --target-report-input "$gap0" \
        --target-max-attempts "$TARGET_MAX_ATTEMPTS" \
        --output "$samples2" \
        --coverage-output "$coverage2"

    require_file "$samples2"
    require_nonempty_file "$coverage2"
    require_nonempty_file "$stage2_parseability_json"
    assert_json "$coverage2" ".grammar_name == \"$grammar_name\"" "coverage stage2 grammar_name mismatch"
    assert_json "$coverage2" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage2 attempts consistency failed"
    assert_json "$stage2_parseability_json" ".grammar_name == \"$grammar_name\" and .summary.attempts == .summary.requested and .summary.accepted <= .summary.requested and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage2 parseability report contract mismatch"

    local attempts2 successes2 covered_rules2 covered_branches2
    attempts2="$(extract_json_u64 "$coverage2" ".sample_attempts")"
    successes2="$(extract_json_u64 "$coverage2" ".sample_successes")"
    covered_rules2="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage2")"
    covered_branches2="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage2")"
    stage2_parseability_attempts="$(parseability_summary_field_u64 "$stage2_parseability_json" "attempts")"
    stage2_parseability_accepted="$(parseability_summary_field_u64 "$stage2_parseability_json" "accepted")"
    stage2_parseability_rejected="$(parseability_summary_field_u64 "$stage2_parseability_json" "rejected")"
    stage2_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage2_parseability_json" "parser_rejections")"
    stage2_parseability_generation_errors="$(parseability_summary_field_u64 "$stage2_parseability_json" "generation_errors")"
    stage2_parseability_empty_generations="$(parseability_summary_field_u64 "$stage2_parseability_json" "empty_generations")"
    stage2_target_drive_alternate_entry_attempts="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_attempts")"
    stage2_target_drive_alternate_entry_accepted_outputs="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_accepted_outputs")"
    stage2_target_drive_alternate_entry_rejected_outputs="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_rejected_outputs")"
    stage2_target_drive_target_timeout_errors="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "target_timeout_errors")"
    stage2_target_drive_helper_timeout_errors="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "helper_timeout_errors")"

    if (( attempts2 < attempts1 )); then
        echo "error: ${label} stage2 sample_attempts regressed ($attempts1 -> $attempts2)" >&2
        exit 1
    fi
    if (( successes2 < successes1 )); then
        echo "error: ${label} stage2 sample_successes regressed ($successes1 -> $successes2)" >&2
        exit 1
    fi
    if (( covered_rules2 < covered_rules1 )); then
        echo "error: ${label} stage2 covered_rules regressed ($covered_rules1 -> $covered_rules2)" >&2
        exit 1
    fi
    if (( covered_branches2 < covered_branches1 )); then
        echo "error: ${label} stage2 covered_branches regressed ($covered_branches1 -> $covered_branches2)" >&2
        exit 1
    fi

    local stage2_log="$LOG_DIR/${label}_stage2_target_drive.log"
    local resolved_targets total_targets target_attempts
    read -r resolved_targets total_targets target_attempts < <(parse_target_summary "$stage2_log")

    if (( total_targets != initial_targets )); then
        echo "error: ${label} stage2 target summary total ($total_targets) does not match stage0 initial targets ($initial_targets)" >&2
        exit 1
    fi
    if (( resolved_targets > total_targets )); then
        echo "error: ${label} stage2 resolved targets exceeds total ($resolved_targets > $total_targets)" >&2
        exit 1
    fi

    run_logged "${label}_stage3_recompute_gap" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count 1 \
        --seed "$((seed_base + 3))" \
        --validate-parseability \
        --parseability-report-json "$stage3_parseability_json" \
        --coverage-input "$coverage2" \
        --output "$samples3" \
        --coverage-output "$coverage3" \
        --gap-report-json "$gap3" \
        --gap-report-threshold "$GAP_THRESHOLD"

    require_nonempty_file "$samples3"
    require_nonempty_file "$coverage3"
    require_nonempty_file "$gap3"
    require_nonempty_file "$stage3_parseability_json"

    assert_json "$coverage3" ".grammar_name == \"$grammar_name\"" "coverage stage3 grammar_name mismatch"
    assert_json "$gap3" ".grammar_name == \"$grammar_name\"" "gap stage3 grammar_name mismatch"
    assert_json "$stage3_parseability_json" ".grammar_name == \"$grammar_name\" and .summary.requested == 1 and .summary.accepted == 1 and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage3 parseability report contract mismatch"

    local final_targets
    final_targets="$(jq -er '.targets | length | numbers' "$gap3")"
    stage3_parseability_attempts="$(parseability_summary_field_u64 "$stage3_parseability_json" "attempts")"
    stage3_parseability_accepted="$(parseability_summary_field_u64 "$stage3_parseability_json" "accepted")"
    stage3_parseability_rejected="$(parseability_summary_field_u64 "$stage3_parseability_json" "rejected")"
    stage3_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage3_parseability_json" "parser_rejections")"
    stage3_parseability_generation_errors="$(parseability_summary_field_u64 "$stage3_parseability_json" "generation_errors")"
    stage3_parseability_empty_generations="$(parseability_summary_field_u64 "$stage3_parseability_json" "empty_generations")"

    if (( final_targets > initial_targets )); then
        echo "error: ${label} final actionable targets regressed ($initial_targets -> $final_targets)" >&2
        exit 1
    fi

    parseability_attempts_total=$((stage0_parseability_attempts + stage1_parseability_attempts + stage2_parseability_attempts + stage3_parseability_attempts))
    parseability_accepted_total=$((stage0_parseability_accepted + stage1_parseability_accepted + stage2_parseability_accepted + stage3_parseability_accepted))
    parseability_rejected_total=$((stage0_parseability_rejected + stage1_parseability_rejected + stage2_parseability_rejected + stage3_parseability_rejected))
    parseability_parser_rejections_total=$((stage0_parseability_parser_rejections + stage1_parseability_parser_rejections + stage2_parseability_parser_rejections + stage3_parseability_parser_rejections))
    parseability_generation_errors_total=$((stage0_parseability_generation_errors + stage1_parseability_generation_errors + stage2_parseability_generation_errors + stage3_parseability_generation_errors))
    parseability_empty_generations_total=$((stage0_parseability_empty_generations + stage1_parseability_empty_generations + stage2_parseability_empty_generations + stage3_parseability_empty_generations))
    parseability_acceptance_rate_total="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_accepted_total" "$parseability_attempts_total")"

    jq -n \
        --arg grammar_name "$grammar_name" \
        --argjson attempts_total "$parseability_attempts_total" \
        --argjson accepted_total "$parseability_accepted_total" \
        --argjson rejected_total "$parseability_rejected_total" \
        --argjson parser_rejections_total "$parseability_parser_rejections_total" \
        --argjson generation_errors_total "$parseability_generation_errors_total" \
        --argjson empty_generations_total "$parseability_empty_generations_total" \
        --argjson acceptance_rate_percent "$parseability_acceptance_rate_total" \
        --argjson alternate_entry_attempts_total "$stage2_target_drive_alternate_entry_attempts" \
        --argjson alternate_entry_accepted_outputs_total "$stage2_target_drive_alternate_entry_accepted_outputs" \
        --argjson alternate_entry_rejected_outputs_total "$stage2_target_drive_alternate_entry_rejected_outputs" \
        --argjson target_timeout_errors_total "$stage2_target_drive_target_timeout_errors" \
        --argjson helper_timeout_errors_total "$stage2_target_drive_helper_timeout_errors" \
        --slurpfile stage0 "$stage0_parseability_json" \
        --slurpfile stage1 "$stage1_parseability_json" \
        --slurpfile stage2 "$stage2_parseability_json" \
        --slurpfile stage3 "$stage3_parseability_json" \
        '{
            grammar_name: $grammar_name,
            parseability_required: true,
            summary: {
                attempts: $attempts_total,
                accepted: $accepted_total,
                rejected: $rejected_total,
                parser_rejections: $parser_rejections_total,
                generation_errors: $generation_errors_total,
                empty_generations: $empty_generations_total,
                acceptance_rate_percent: $acceptance_rate_percent
            },
            target_drive_validation: {
                alternate_entry_attempts_total: $alternate_entry_attempts_total,
                alternate_entry_accepted_outputs_total: $alternate_entry_accepted_outputs_total,
                alternate_entry_rejected_outputs_total: $alternate_entry_rejected_outputs_total,
                target_timeout_errors_total: $target_timeout_errors_total,
                helper_timeout_errors_total: $helper_timeout_errors_total
            },
            stages: {
                stage0_baseline: $stage0[0],
                stage1_gap_priority: $stage1[0],
                stage2_target_drive: $stage2[0],
                stage3_recompute_gap: $stage3[0]
            }
        }' >"$parseability_report_json"
    require_nonempty_file "$parseability_report_json"

    echo "    ${label} closed-loop summary: initial_targets=$initial_targets resolved=$resolved_targets final_targets=$final_targets target_attempts=$target_attempts parseability_attempts=$parseability_attempts_total accepted=$parseability_accepted_total alternate_entry_attempts=$stage2_target_drive_alternate_entry_attempts target_timeout_errors=$stage2_target_drive_target_timeout_errors helper_timeout_errors=$stage2_target_drive_helper_timeout_errors"
    echo "${label},${grammar_name},${SAMPLE_COUNT},${seed_base},${parseability_attempts_total},${parseability_accepted_total},${parseability_rejected_total},${parseability_parser_rejections_total},${parseability_generation_errors_total},${parseability_empty_generations_total},${parseability_acceptance_rate_total},${parseability_report_json},${stage2_target_drive_alternate_entry_attempts},${stage2_target_drive_alternate_entry_accepted_outputs},${stage2_target_drive_alternate_entry_rejected_outputs},${stage2_target_drive_target_timeout_errors},${stage2_target_drive_helper_timeout_errors},${initial_targets},${resolved_targets},${final_targets},${target_attempts},${attempts0},${attempts1},${attempts2},${successes0},${successes1},${successes2},pass" >>"$SUMMARY_CSV"
}

require_tool jq
require_tool perl
require_file "$RETURN_JSON"
require_file "$SEMANTIC_JSON"

echo "==> Annotation stimuli quality gate"
echo "state_dir: $STATE_DIR"
echo "sample_count: $SAMPLE_COUNT"
echo "gap_threshold: $GAP_THRESHOLD"
echo "target_max_attempts: $TARGET_MAX_ATTEMPTS"
echo "filter: $FILTER"
echo "return_seed_base: $RETURN_SEED_BASE"
echo "semantic_seed_base: $SEMANTIC_SEED_BASE"

echo "grammar,grammar_name,sample_count,seed_base,parseability_attempts_total,parseability_accepted_total,parseability_rejected_total,parseability_parser_rejections_total,parseability_generation_errors_total,parseability_empty_generations_total,parseability_acceptance_rate_percent,parseability_report_json,target_drive_alternate_entry_attempts_total,target_drive_alternate_entry_accepted_outputs_total,target_drive_alternate_entry_rejected_outputs_total,target_drive_target_timeout_errors_total,target_drive_helper_timeout_errors_total,initial_targets,resolved_targets,final_targets,target_attempts,stage0_sample_attempts,stage1_sample_attempts,stage2_sample_attempts,stage0_sample_successes,stage1_sample_successes,stage2_sample_successes,status" >"$SUMMARY_CSV"

run_logged_rust "build_generated_ast_pipeline" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

if [[ "$FILTER" == "all" || "$FILTER" == "return" ]]; then
    closed_loop_for_grammar "return" "return_annotation" "$RETURN_JSON" "$RETURN_SEED_BASE"
fi
if [[ "$FILTER" == "all" || "$FILTER" == "semantic" ]]; then
    closed_loop_for_grammar "semantic" "semantic_annotation" "$SEMANTIC_JSON" "$SEMANTIC_SEED_BASE"
fi

{
    echo "PGEN Annotation Stimuli Quality Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sample_count: $SAMPLE_COUNT"
    echo "gap_threshold: $GAP_THRESHOLD"
    echo "target_max_attempts: $TARGET_MAX_ATTEMPTS"
    echo "filter: $FILTER"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ Annotation stimuli quality gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
