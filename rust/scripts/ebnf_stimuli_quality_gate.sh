#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"
STATE_DIR="${PGEN_EBNF_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/ebnf_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SAMPLE_COUNT="${PGEN_EBNF_STIMULI_QUALITY_COUNT:-12}"
GAP_THRESHOLD="${PGEN_EBNF_STIMULI_QUALITY_GAP_THRESHOLD:-1}"
TARGET_MAX_ATTEMPTS="${PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS:-5000}"
CONTRACT_FILE="${PGEN_EBNF_STIMULI_QUALITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/ebnf_stimuli_contract.json}"
FRONTEND_IMPL="${PGEN_EBNF_FRONTEND_IMPL:-rust}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"
EBNF_BOOTSTRAP_GRAMMAR="$GRAMMARS_DIR/ebnf.ebnf"
EBNF_BOOTSTRAP_JSON="$WORK_DIR/ebnf_bootstrap.json"
EBNF_BOOTSTRAP_RS="$WORK_DIR/ebnf_bootstrap.rs"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_EBNF_STIMULI_QUALITY_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$GAP_THRESHOLD" =~ ^[0-9]+$ ]] || [[ "$GAP_THRESHOLD" -lt 1 ]]; then
    echo "error: PGEN_EBNF_STIMULI_QUALITY_GAP_THRESHOLD must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
if [[ "$FRONTEND_IMPL" != "perl" && "$FRONTEND_IMPL" != "rust" ]]; then
    echo "error: PGEN_EBNF_FRONTEND_IMPL must be 'perl' or 'rust'" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

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
        tail -n 80 "$log_file" >&2 || true
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
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

extract_json_u64() {
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
    jq -er '.summary | if .attempts == 0 then 0 else ((.accepted * 100.0) / .attempts) end' "$path"
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
    local require_parseability="$5"

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
    local parseability_report_json="$WORK_DIR/${label}_parseability_report.json"
    local stage0_parseability_json="$WORK_DIR/${label}_parseability_stage0.json"
    local stage1_parseability_json="$WORK_DIR/${label}_parseability_stage1.json"
    local stage2_parseability_json="$WORK_DIR/${label}_parseability_stage2.json"
    local stage3_parseability_json="$WORK_DIR/${label}_parseability_stage3.json"
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
    local parseability_report_path_for_summary="n/a"

    local -a parseability_args_stage0=()
    local -a parseability_args_stage1=()
    local -a parseability_args_stage2=()
    local -a parseability_args_stage3=()
    if [[ "$require_parseability" -eq 1 ]]; then
        parseability_args_stage0=(--validate-parseability --parseability-report-json "$stage0_parseability_json")
        parseability_args_stage1=(--validate-parseability --parseability-report-json "$stage1_parseability_json")
        parseability_args_stage2=(--validate-parseability --parseability-report-json "$stage2_parseability_json")
        parseability_args_stage3=(--validate-parseability --parseability-report-json "$stage3_parseability_json")
    fi

    run_logged "${label}_stage0_baseline" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$seed_base" \
        "${parseability_args_stage0[@]}" \
        --gap-report-threshold "$GAP_THRESHOLD" \
        --output "$samples0" \
        --coverage-output "$coverage0" \
        --gap-report-json "$gap0"

    require_nonempty_file "$samples0"
    require_nonempty_file "$coverage0"
    require_nonempty_file "$gap0"

    assert_json "$coverage0" ".grammar_name == \"$grammar_name\"" "coverage stage0 grammar_name mismatch"
    assert_json "$coverage0" ".total_rules > 0" "coverage stage0 must report total_rules > 0"
    assert_json "$coverage0" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage0 attempts consistency failed"
    assert_json "$coverage0" ".sample_successes >= $SAMPLE_COUNT" "coverage stage0 sample_successes below requested count"

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
    if [[ "$require_parseability" -eq 1 ]]; then
        require_nonempty_file "$stage0_parseability_json"
        assert_json "$stage0_parseability_json" ".grammar_name == \"$grammar_name\"" "stage0 parseability grammar_name mismatch"
        stage0_parseability_attempts="$(parseability_summary_field_u64 "$stage0_parseability_json" "attempts")"
        stage0_parseability_accepted="$(parseability_summary_field_u64 "$stage0_parseability_json" "accepted")"
        stage0_parseability_rejected="$(parseability_summary_field_u64 "$stage0_parseability_json" "rejected")"
        stage0_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage0_parseability_json" "parser_rejections")"
        stage0_parseability_generation_errors="$(parseability_summary_field_u64 "$stage0_parseability_json" "generation_errors")"
        stage0_parseability_empty_generations="$(parseability_summary_field_u64 "$stage0_parseability_json" "empty_generations")"
    fi

    run_logged "${label}_stage1_gap_priority" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$((seed_base + 1))" \
        "${parseability_args_stage1[@]}" \
        --coverage-input "$coverage0" \
        --gap-priority-report-input "$gap0" \
        --output "$samples1" \
        --coverage-output "$coverage1" \
        --gap-report-json "$gap1" \
        --gap-report-threshold "$GAP_THRESHOLD"

    require_nonempty_file "$samples1"
    require_nonempty_file "$coverage1"
    require_nonempty_file "$gap1"

    assert_json "$coverage1" ".grammar_name == \"$grammar_name\"" "coverage stage1 grammar_name mismatch"
    assert_json "$coverage1" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage1 attempts consistency failed"
    assert_json "$gap1" ".grammar_name == \"$grammar_name\"" "gap stage1 grammar_name mismatch"

    local attempts1 successes1 covered_rules1 covered_branches1
    attempts1="$(extract_json_u64 "$coverage1" ".sample_attempts")"
    successes1="$(extract_json_u64 "$coverage1" ".sample_successes")"
    covered_rules1="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage1")"
    covered_branches1="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage1")"

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
    if [[ "$require_parseability" -eq 1 ]]; then
        require_nonempty_file "$stage1_parseability_json"
        assert_json "$stage1_parseability_json" ".grammar_name == \"$grammar_name\"" "stage1 parseability grammar_name mismatch"
        stage1_parseability_attempts="$(parseability_summary_field_u64 "$stage1_parseability_json" "attempts")"
        stage1_parseability_accepted="$(parseability_summary_field_u64 "$stage1_parseability_json" "accepted")"
        stage1_parseability_rejected="$(parseability_summary_field_u64 "$stage1_parseability_json" "rejected")"
        stage1_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage1_parseability_json" "parser_rejections")"
        stage1_parseability_generation_errors="$(parseability_summary_field_u64 "$stage1_parseability_json" "generation_errors")"
        stage1_parseability_empty_generations="$(parseability_summary_field_u64 "$stage1_parseability_json" "empty_generations")"
    fi

    run_logged "${label}_stage2_target_drive" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --seed "$((seed_base + 2))" \
        "${parseability_args_stage2[@]}" \
        --coverage-input "$coverage1" \
        --target-report-input "$gap0" \
        --target-max-attempts "$TARGET_MAX_ATTEMPTS" \
        --output "$samples2" \
        --coverage-output "$coverage2"

    require_file "$samples2"
    require_nonempty_file "$coverage2"
    assert_json "$coverage2" ".grammar_name == \"$grammar_name\"" "coverage stage2 grammar_name mismatch"
    assert_json "$coverage2" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage2 attempts consistency failed"

    local attempts2 successes2 covered_rules2 covered_branches2
    attempts2="$(extract_json_u64 "$coverage2" ".sample_attempts")"
    successes2="$(extract_json_u64 "$coverage2" ".sample_successes")"
    covered_rules2="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage2")"
    covered_branches2="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage2")"

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
    if [[ "$require_parseability" -eq 1 ]]; then
        require_nonempty_file "$stage2_parseability_json"
        assert_json "$stage2_parseability_json" ".grammar_name == \"$grammar_name\"" "stage2 parseability grammar_name mismatch"
        stage2_parseability_attempts="$(parseability_summary_field_u64 "$stage2_parseability_json" "attempts")"
        stage2_parseability_accepted="$(parseability_summary_field_u64 "$stage2_parseability_json" "accepted")"
        stage2_parseability_rejected="$(parseability_summary_field_u64 "$stage2_parseability_json" "rejected")"
        stage2_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage2_parseability_json" "parser_rejections")"
        stage2_parseability_generation_errors="$(parseability_summary_field_u64 "$stage2_parseability_json" "generation_errors")"
        stage2_parseability_empty_generations="$(parseability_summary_field_u64 "$stage2_parseability_json" "empty_generations")"
        stage2_target_drive_alternate_entry_attempts="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_attempts")"
        stage2_target_drive_alternate_entry_accepted_outputs="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_accepted_outputs")"
        stage2_target_drive_alternate_entry_rejected_outputs="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_rejected_outputs")"
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
        "${parseability_args_stage3[@]}" \
        --coverage-input "$coverage2" \
        --output "$samples3" \
        --coverage-output "$coverage3" \
        --gap-report-json "$gap3" \
        --gap-report-threshold "$GAP_THRESHOLD"

    require_nonempty_file "$samples3"
    require_nonempty_file "$coverage3"
    require_nonempty_file "$gap3"

    assert_json "$coverage3" ".grammar_name == \"$grammar_name\"" "coverage stage3 grammar_name mismatch"
    assert_json "$coverage3" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage3 attempts consistency failed"
    assert_json "$gap3" ".grammar_name == \"$grammar_name\"" "gap stage3 grammar_name mismatch"

    local successes3 final_targets
    successes3="$(extract_json_u64 "$coverage3" ".sample_successes")"
    final_targets="$(jq -er '.targets | length | numbers' "$gap3")"
    if [[ "$require_parseability" -eq 1 ]]; then
        require_nonempty_file "$stage3_parseability_json"
        assert_json "$stage3_parseability_json" ".grammar_name == \"$grammar_name\"" "stage3 parseability grammar_name mismatch"
        stage3_parseability_attempts="$(parseability_summary_field_u64 "$stage3_parseability_json" "attempts")"
        stage3_parseability_accepted="$(parseability_summary_field_u64 "$stage3_parseability_json" "accepted")"
        stage3_parseability_rejected="$(parseability_summary_field_u64 "$stage3_parseability_json" "rejected")"
        stage3_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage3_parseability_json" "parser_rejections")"
        stage3_parseability_generation_errors="$(parseability_summary_field_u64 "$stage3_parseability_json" "generation_errors")"
        stage3_parseability_empty_generations="$(parseability_summary_field_u64 "$stage3_parseability_json" "empty_generations")"
    fi

    if (( final_targets > initial_targets )); then
        echo "error: ${label} final actionable targets regressed ($initial_targets -> $final_targets)" >&2
        exit 1
    fi

    if [[ "$require_parseability" -eq 1 ]]; then
        parseability_attempts_total=$((stage0_parseability_attempts + stage1_parseability_attempts + stage2_parseability_attempts + stage3_parseability_attempts))
        parseability_accepted_total=$((stage0_parseability_accepted + stage1_parseability_accepted + stage2_parseability_accepted + stage3_parseability_accepted))
        parseability_rejected_total=$((stage0_parseability_rejected + stage1_parseability_rejected + stage2_parseability_rejected + stage3_parseability_rejected))
        parseability_parser_rejections_total=$((stage0_parseability_parser_rejections + stage1_parseability_parser_rejections + stage2_parseability_parser_rejections + stage3_parseability_parser_rejections))
        parseability_generation_errors_total=$((stage0_parseability_generation_errors + stage1_parseability_generation_errors + stage2_parseability_generation_errors + stage3_parseability_generation_errors))
        parseability_empty_generations_total=$((stage0_parseability_empty_generations + stage1_parseability_empty_generations + stage2_parseability_empty_generations + stage3_parseability_empty_generations))
        parseability_acceptance_rate_total="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_accepted_total" "$parseability_attempts_total")"
        jq -n \
            --arg label "$label" \
            --arg grammar_name "$grammar_name" \
            --argjson required "$require_parseability" \
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
            --slurpfile stage0 "$stage0_parseability_json" \
            --slurpfile stage1 "$stage1_parseability_json" \
            --slurpfile stage2 "$stage2_parseability_json" \
            --slurpfile stage3 "$stage3_parseability_json" \
            '{
                id: $label,
                grammar_name: $grammar_name,
                parseability_required: ($required == 1),
                stages: {
                    stage0_baseline: $stage0[0],
                    stage1_gap_priority: $stage1[0],
                    stage2_target_drive: $stage2[0],
                    stage3_recompute_gap: $stage3[0]
                },
                totals: {
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
                    alternate_entry_rejected_outputs_total: $alternate_entry_rejected_outputs_total
                }
            }' >"$parseability_report_json"
        parseability_report_path_for_summary="$parseability_report_json"
    fi

    echo "    ${label} closed-loop summary: parseability_required=${require_parseability} initial_targets=$initial_targets resolved=$resolved_targets final_targets=$final_targets target_attempts=$target_attempts alternate_entry_attempts=$stage2_target_drive_alternate_entry_attempts"
    echo "${label},${grammar_name},${require_parseability},${parseability_attempts_total},${parseability_accepted_total},${parseability_rejected_total},${parseability_parser_rejections_total},${parseability_generation_errors_total},${parseability_empty_generations_total},${parseability_acceptance_rate_total},${parseability_report_path_for_summary},${stage2_target_drive_alternate_entry_attempts},${stage2_target_drive_alternate_entry_accepted_outputs},${stage2_target_drive_alternate_entry_rejected_outputs},${initial_targets},${resolved_targets},${final_targets},${target_attempts},${successes0},${successes3},pass" >>"$SUMMARY_CSV"
}

require_tool jq
require_tool perl
require_tool base64
require_file "$CONTRACT_FILE"

grammar_count="$(jq -er '.grammars | length | numbers' "$CONTRACT_FILE")"
contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
require_ebnf_parseability="$(jq -er '[.grammars[] | select(.grammar_name == "ebnf" and .require_parseability == true)] | if length > 0 then 1 else 0 end' "$CONTRACT_FILE")"
if [[ "$grammar_count" -lt 1 ]]; then
    echo "error: contract '$CONTRACT_FILE' must contain at least one grammar entry" >&2
    exit 1
fi
if [[ "$FRONTEND_IMPL" == "perl" || "$require_ebnf_parseability" -eq 1 ]]; then
    require_file "$EBNF_TO_JSON"
fi

echo "==> EBNF stimuli quality gate"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_count: $grammar_count"
echo "sample_count: $SAMPLE_COUNT"
echo "gap_threshold: $GAP_THRESHOLD"
echo "target_max_attempts: $TARGET_MAX_ATTEMPTS"
echo "frontend_impl: $FRONTEND_IMPL"
echo "require_ebnf_parseability: $require_ebnf_parseability"

echo "grammar,grammar_name,parseability_required,parseability_attempts_total,parseability_accepted_total,parseability_rejected_total,parseability_parser_rejections_total,parseability_generation_errors_total,parseability_empty_generations_total,parseability_acceptance_rate_percent,parseability_report_json,target_drive_alternate_entry_attempts_total,target_drive_alternate_entry_accepted_outputs_total,target_drive_alternate_entry_rejected_outputs_total,initial_targets,resolved_targets,final_targets,target_attempts,stage0_successes,stage3_successes,status" >"$SUMMARY_CSV"

run_logged_rust "build_generated_ast_pipeline" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

run_frontend_to_json() {
    local grammar_file="$1"
    local json_out="$2"
    if [[ "$FRONTEND_IMPL" == "perl" ]]; then
        "$EBNF_TO_JSON" --pretty --quiet "$grammar_file" -o "$json_out"
    else
        "$AST_PIPELINE_BIN" "$grammar_file" --emit-raw-ast-json "$json_out"
    fi
}

if [[ "$require_ebnf_parseability" -eq 1 ]]; then
    require_file "$EBNF_BOOTSTRAP_GRAMMAR"

    run_logged "prepare_ebnf_frontend_json_for_parseability" \
        "$EBNF_TO_JSON" --pretty --quiet "$EBNF_BOOTSTRAP_GRAMMAR" -o "$EBNF_BOOTSTRAP_JSON"
    require_nonempty_file "$EBNF_BOOTSTRAP_JSON"

    run_logged "prepare_ebnf_generated_parser_for_parseability" \
        "$AST_PIPELINE_BIN" "$EBNF_BOOTSTRAP_JSON" \
        --generate-parser \
        --eliminate-left-recursion \
        --output "$EBNF_BOOTSTRAP_RS"
    require_nonempty_file "$EBNF_BOOTSTRAP_RS"

    run_logged_rust "rebuild_generated_ast_pipeline_with_ebnf_dual_run" \
        env PGEN_EBNF_PARSER_PATH="$EBNF_BOOTSTRAP_RS" \
        cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

    if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
        echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after ebnf_dual_run rebuild" >&2
        exit 1
    fi
elif [[ "$FRONTEND_IMPL" == "rust" ]]; then
    run_logged_rust "rebuild_generated_ast_pipeline_with_ebnf_dual_run" \
        cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

    if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
        echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after ebnf_dual_run rebuild" >&2
        exit 1
    fi
fi

mapfile -t grammar_rows < <(jq -r '.grammars[] | @base64' "$CONTRACT_FILE")

for encoded in "${grammar_rows[@]}"; do
    decoded="$(printf '%s' "$encoded" | base64 --decode)"
    label="$(printf '%s\n' "$decoded" | jq -er '.id | strings')"
    grammar_name="$(printf '%s\n' "$decoded" | jq -er '.grammar_name | strings')"
    ebnf_path_rel="$(printf '%s\n' "$decoded" | jq -er '.ebnf_path | strings')"
    seed_base="$(printf '%s\n' "$decoded" | jq -er '.seed_base | numbers')"
    parseability_required="$(printf '%s\n' "$decoded" | jq -er 'if .require_parseability then 1 else 0 end')"

    ebnf_path="$ROOT_DIR/$ebnf_path_rel"
    grammar_json="$WORK_DIR/${label}.json"
    parser_out="$WORK_DIR/${label}_parser.rs"

    require_file "$ebnf_path"

    run_logged "${label}_frontend_ebnf_to_json" \
        run_frontend_to_json "$ebnf_path" "$grammar_json"

    require_nonempty_file "$grammar_json"
    assert_json "$grammar_json" ".grammar_name == \"$grammar_name\"" "frontend grammar_name mismatch for contract entry '$label'"
    assert_json "$grammar_json" ".raw_ast | type == \"array\"" "frontend output raw_ast must be an array for contract entry '$label'"

    run_logged "${label}_generate_parser" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-parser \
        --eliminate-left-recursion \
        --output "$parser_out"
    require_nonempty_file "$parser_out"

    closed_loop_for_grammar "$label" "$grammar_name" "$grammar_json" "$seed_base" "$parseability_required"
done

{
    echo "PGEN EBNF Stimuli Quality Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ EBNF stimuli quality gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
