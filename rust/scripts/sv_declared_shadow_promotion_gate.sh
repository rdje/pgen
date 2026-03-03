#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_DECLARED_SHADOW_PROMOTION_STATE_DIR:-$RUST_DIR/target/sv_declared_shadow_promotion_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_TXT="$STATE_DIR/summary.txt"
PROMOTION_REPORT_JSON="$WORK_DIR/systemverilog_declared_identifier_promotion_report.json"

PROMOTION_MODE="${PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE:-auto}" # auto|0|1
TRIALS="${PGEN_SV_DECLARED_SHADOW_PROMOTION_TRIALS:-3}"
SAMPLE_COUNT="${PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT:-6}"
SEED_BASE="${PGEN_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE:-12001}"
TARGET_MAX_ATTEMPTS="${PGEN_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS:-400}"
PARSE_FULL_MODE="${PGEN_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE:-auto}" # auto|0|1
MIN_CHECKED="${PGEN_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED:-2}"
SEMANTIC_CLOSURE_MODE="${PGEN_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE:-1}" # 0|1
PROMOTION_STIMULI_MODE="${PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE:-sv_file}"
DECLARED_SHADOW_PARSEABLE_ONLY="${PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY:-1}" # 0|1

if [[ "$PROMOTION_MODE" != "auto" && "$PROMOTION_MODE" != "0" && "$PROMOTION_MODE" != "1" ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$TRIALS" =~ ^[0-9]+$ ]] || [[ "$TRIALS" -lt 1 ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_TRIALS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$MIN_CHECKED" =~ ^[0-9]+$ ]] || [[ "$MIN_CHECKED" -lt 1 ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SEMANTIC_CLOSURE_MODE" =~ ^[01]$ ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE must be 0 or 1" >&2
    exit 2
fi
if [[ "$PROMOTION_STIMULI_MODE" != "sv_file" && "$PROMOTION_STIMULI_MODE" != "sv_snippet" && "$PROMOTION_STIMULI_MODE" != "sv_pp_file" && "$PROMOTION_STIMULI_MODE" != "sv_pp_snippet" && "$PROMOTION_STIMULI_MODE" != "sv_semantic_file" ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE must be one of: sv_file, sv_snippet, sv_pp_file, sv_pp_snippet, sv_semantic_file" >&2
    exit 2
fi
if ! [[ "$DECLARED_SHADOW_PARSEABLE_ONLY" =~ ^[01]$ ]]; then
    echo "error: PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY must be 0 or 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

echo "==> SV declared-identifier shadow promotion gate"
echo "state_dir: $STATE_DIR"
echo "promotion_mode: $PROMOTION_MODE"
echo "trials: $TRIALS"
echo "sample_count: $SAMPLE_COUNT"
echo "seed_base: $SEED_BASE"
echo "target_max_attempts: $TARGET_MAX_ATTEMPTS"
echo "parse_full_mode: $PARSE_FULL_MODE"
echo "semantic_closure_mode: $SEMANTIC_CLOSURE_MODE"
echo "promotion_stimuli_mode: $PROMOTION_STIMULI_MODE"
echo "declared_shadow_parseable_only: $DECLARED_SHADOW_PARSEABLE_ONLY"
echo "min_checked: $MIN_CHECKED"

if [[ "$PROMOTION_MODE" == "0" ]]; then
    jq -n \
        --arg mode "$PROMOTION_MODE" \
        --arg note "promotion gate disabled by mode" \
        '{
            mode: $mode,
            status: "skipped",
            note: $note,
            eligible_for_runtime_enforcement: false
        }' >"$PROMOTION_REPORT_JSON"
    {
        echo "SV declared-identifier shadow promotion gate: skipped"
        echo "mode: $PROMOTION_MODE"
        echo "note: promotion gate disabled by mode"
        echo "report_json: $PROMOTION_REPORT_JSON"
    } >"$SUMMARY_TXT"
    cat "$SUMMARY_TXT"
    exit 0
fi

TRIAL_CASES_JSONL="$WORK_DIR/trial_cases.jsonl"
: >"$TRIAL_CASES_JSONL"

total_checked=0
total_passed=0
total_failed=0
trial_passed=0
trial_failed=0
trial_missing_report=0
trial_gate_failures=0

classify_trial_gate_blocker() {
    local trial_log="$1"
    local trial_status="$2"
    local report_present="$3"
    local checked="$4"
    local failed="$5"
    local blocker_key="unknown_gate_failure"
    local blocker_detail=""

    if [[ "$trial_status" == "pass" ]]; then
        blocker_key="none"
    elif [[ "$report_present" -eq 0 ]]; then
        blocker_key="shadow_report_unavailable"
        blocker_detail="declared shadow report missing"
    elif [[ "$trial_status" == "shadow_fail" ]]; then
        blocker_key="declared_identifier_shadow_violation"
        blocker_detail="failed=${failed}/${checked}"
    elif grep -q "strict declared-identifier shadow mode requires at least one parseable sample" "$trial_log"; then
        blocker_key="no_parseable_shadow_samples"
    elif grep -q "semantic baseline validation failed" "$trial_log"; then
        blocker_key="semantic_baseline_validation_failed"
    elif grep -q "declared identifier contract suite failed" "$trial_log"; then
        blocker_key="declared_identifier_contract_suite_failed"
    elif grep -q "width compatibility contract suite failed" "$trial_log"; then
        blocker_key="width_compatibility_contract_suite_failed"
    elif grep -q "port binding legality contract suite failed" "$trial_log"; then
        blocker_key="port_binding_legality_contract_suite_failed"
    elif grep -q "package qualification contract suite failed" "$trial_log"; then
        blocker_key="package_qualification_contract_suite_failed"
    elif grep -q "context legality contract suite failed" "$trial_log"; then
        blocker_key="context_legality_contract_suite_failed"
    elif grep -q "strict parse_full mode requested but parseability probe did not expose adapter support" "$trial_log"; then
        blocker_key="parse_full_adapter_unavailable"
    elif grep -q "strict parse_full quality mode requires parse_full quality report availability" "$trial_log"; then
        blocker_key="parse_full_quality_report_unavailable"
    else
        local failed_stage=""
        failed_stage="$(awk '/^==> /{stage=$2} /^    fail \(/ {failed=stage} END{if(failed!="") print failed}' "$trial_log" || true)"
        if [[ -n "$failed_stage" ]]; then
            blocker_key="stage_failure"
            blocker_detail="$failed_stage"
        fi
    fi

    if [[ "$blocker_key" != "none" && -z "$blocker_detail" ]]; then
        blocker_detail="$(sed -nE 's/^error: (.*)$/\1/p' "$trial_log" | tail -n 1 || true)"
    fi

    printf '%s\t%s\n' "$blocker_key" "$blocker_detail"
}

for ((trial_idx = 0; trial_idx < TRIALS; trial_idx++)); do
    trial_seed_base=$((SEED_BASE + (trial_idx * 100000)))
    trial_state_dir="$WORK_DIR/trial_${trial_idx}"
    trial_log="$LOG_DIR/trial_${trial_idx}.log"
    mkdir -p "$trial_state_dir"

    echo "==> strict_trial_${trial_idx}"
    trial_exit=0
    if (
        cd "$RUST_DIR"
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$trial_state_dir" \
            PGEN_SV_STIMULI_QUALITY_COUNT="$SAMPLE_COUNT" \
            PGEN_SV_STIMULI_QUALITY_SEED_BASE="$trial_seed_base" \
            PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="$TARGET_MAX_ATTEMPTS" \
            PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE="$PARSE_FULL_MODE" \
            PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE="$SEMANTIC_CLOSURE_MODE" \
            PGEN_SV_STIMULI_QUALITY_MODE="$PROMOTION_STIMULI_MODE" \
            PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE=1 \
            PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY="$DECLARED_SHADOW_PARSEABLE_ONLY" \
            PGEN_SV_STIMULI_DIFF_MODE=0 \
            PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
            ./scripts/sv_stimuli_quality_gate.sh
    ) >"$trial_log" 2>&1; then
        echo "    ok (${trial_log})"
    else
        trial_exit=$?
        echo "    fail (${trial_log})"
    fi

    shadow_report_json="$trial_state_dir/work/systemverilog_declared_identifier_shadow_report.json"
    report_present=0
    checked=0
    passed=0
    failed=0
    effective_mode="missing_report"
    trial_note="declared shadow report missing"
    trial_status="infra_error"

    if [[ -f "$shadow_report_json" ]]; then
        report_present=1
        checked="$(jq -er '(.totals.checked // 0) | numbers' "$shadow_report_json" 2>/dev/null || echo 0)"
        passed="$(jq -er '(.totals.passed // 0) | numbers' "$shadow_report_json" 2>/dev/null || echo 0)"
        failed="$(jq -er '(.totals.failed // 0) | numbers' "$shadow_report_json" 2>/dev/null || echo 0)"
        effective_mode="$(jq -er '.effective_mode // "unknown"' "$shadow_report_json" 2>/dev/null || echo "unknown")"
        trial_note="$(jq -er '.note // ""' "$shadow_report_json" 2>/dev/null || echo "")"
        if [[ -z "$trial_note" ]]; then
            trial_note="declared shadow report note unavailable"
        fi
        total_checked=$((total_checked + checked))
        total_passed=$((total_passed + passed))
        total_failed=$((total_failed + failed))
    fi

    if [[ "$report_present" -eq 0 ]]; then
        trial_missing_report=$((trial_missing_report + 1))
        trial_status="infra_error"
    elif [[ "$trial_exit" -eq 0 && "$failed" -eq 0 ]]; then
        trial_status="pass"
        trial_passed=$((trial_passed + 1))
    elif [[ "$failed" -gt 0 ]]; then
        trial_status="shadow_fail"
        trial_failed=$((trial_failed + 1))
    else
        trial_status="gate_fail"
        trial_gate_failures=$((trial_gate_failures + 1))
    fi

    trial_blocker_key="unknown_gate_failure"
    trial_blocker_detail=""
    IFS=$'\t' read -r trial_blocker_key trial_blocker_detail <<<"$(classify_trial_gate_blocker "$trial_log" "$trial_status" "$report_present" "$checked" "$failed")"

    jq -n \
        --argjson trial_index "$trial_idx" \
        --argjson seed_base "$trial_seed_base" \
        --arg log_file "$trial_log" \
        --arg shadow_report_json "$shadow_report_json" \
        --argjson report_present "$report_present" \
        --argjson exit_code "$trial_exit" \
        --arg status "$trial_status" \
        --arg note "$trial_note" \
        --arg effective_mode "$effective_mode" \
        --arg blocker_key "$trial_blocker_key" \
        --arg blocker_detail "$trial_blocker_detail" \
        --argjson checked "$checked" \
        --argjson passed "$passed" \
        --argjson failed "$failed" \
        '{
            trial_index: $trial_index,
            seed_base: $seed_base,
            log_file: $log_file,
            shadow_report_json: $shadow_report_json,
            report_present: $report_present,
            exit_code: $exit_code,
            status: $status,
            note: $note,
            effective_mode: $effective_mode,
            blocker_key: $blocker_key,
            blocker_detail: $blocker_detail,
            totals: {
                checked: $checked,
                passed: $passed,
                failed: $failed
            }
        }' >>"$TRIAL_CASES_JSONL"
done

trial_cases_json="$(jq -s '.' "$TRIAL_CASES_JSONL")"
blocker_breakdown_json="$(jq -cn --argjson trials "$trial_cases_json" '
    $trials
    | map(select(.status != "pass") | {key: .blocker_key})
    | sort_by(.key)
    | group_by(.key)
    | map({key: .[0].key, count: length})
')"
non_shadow_blocker_breakdown_json="$(jq -cn --argjson trials "$trial_cases_json" '
    $trials
    | map(select(.status == "gate_fail" or .status == "infra_error") | {key: .blocker_key})
    | sort_by(.key)
    | group_by(.key)
    | map({key: .[0].key, count: length})
')"
primary_non_shadow_blocker="$(jq -r '
    if length == 0 then
        "none"
    else
        max_by(.count).key
    end
' <<<"$non_shadow_blocker_breakdown_json")"
eligible_for_runtime_enforcement=0
promotion_recommendation="hold"
promotion_note=""

if [[ "$trial_gate_failures" -gt 0 || "$trial_missing_report" -gt 0 ]]; then
    if [[ "$primary_non_shadow_blocker" != "none" ]]; then
        promotion_note="one or more strict shadow trials failed for non-shadow reasons (primary blocker: ${primary_non_shadow_blocker})"
    else
        promotion_note="one or more strict shadow trials failed for non-shadow reasons"
    fi
elif [[ "$total_checked" -lt "$MIN_CHECKED" ]]; then
    promotion_note="insufficient checked samples (${total_checked} < ${MIN_CHECKED})"
elif [[ "$total_failed" -eq 0 && "$trial_failed" -eq 0 && "$trial_gate_failures" -eq 0 ]]; then
    eligible_for_runtime_enforcement=1
    promotion_recommendation="enable_runtime_declared_identifiers"
    promotion_note="all strict shadow trials passed with zero failures"
else
    promotion_note="strict shadow trials still report failures (${total_failed}/${total_checked})"
fi

jq -n \
    --arg mode "$PROMOTION_MODE" \
    --arg recommendation "$promotion_recommendation" \
    --arg note "$promotion_note" \
    --arg promotion_stimuli_mode "$PROMOTION_STIMULI_MODE" \
    --argjson declared_shadow_parseable_only "$DECLARED_SHADOW_PARSEABLE_ONLY" \
    --argjson trials "$TRIALS" \
    --argjson sample_count "$SAMPLE_COUNT" \
    --argjson min_checked "$MIN_CHECKED" \
    --arg parse_full_mode "$PARSE_FULL_MODE" \
    --argjson semantic_closure_mode "$SEMANTIC_CLOSURE_MODE" \
    --argjson total_checked "$total_checked" \
    --argjson total_passed "$total_passed" \
    --argjson total_failed "$total_failed" \
    --argjson trial_passed "$trial_passed" \
    --argjson trial_failed "$trial_failed" \
    --argjson trial_gate_failures "$trial_gate_failures" \
    --argjson trial_missing_report "$trial_missing_report" \
    --arg primary_non_shadow_blocker "$primary_non_shadow_blocker" \
    --argjson blocker_breakdown "$blocker_breakdown_json" \
    --argjson non_shadow_blocker_breakdown "$non_shadow_blocker_breakdown_json" \
    --argjson eligible "$eligible_for_runtime_enforcement" \
    --argjson trial_cases "$trial_cases_json" \
    '{
        mode: $mode,
        status: "completed",
        recommendation: $recommendation,
        note: $note,
        promotion_stimuli_mode: $promotion_stimuli_mode,
        declared_shadow_parseable_only: $declared_shadow_parseable_only,
        parse_full_mode: $parse_full_mode,
        semantic_closure_mode: $semantic_closure_mode,
        eligibility: {
            eligible_for_runtime_enforcement: ($eligible == 1),
            min_checked: $min_checked
        },
        totals: {
            trials: $trials,
            sample_count_per_trial: $sample_count,
            checked: $total_checked,
            passed: $total_passed,
            failed: $total_failed,
            trial_passed: $trial_passed,
            trial_failed: $trial_failed,
            trial_gate_failures: $trial_gate_failures,
            trial_missing_report: $trial_missing_report
        },
        blockers: {
            failed_trial_count: ($trial_failed + $trial_gate_failures + $trial_missing_report),
            non_shadow_blocked_trial_count: ($trial_gate_failures + $trial_missing_report),
            primary_non_shadow_blocker: $primary_non_shadow_blocker,
            breakdown: $blocker_breakdown,
            non_shadow_breakdown: $non_shadow_blocker_breakdown
        },
        trials: $trial_cases
    }' >"$PROMOTION_REPORT_JSON"

{
    echo "SV declared-identifier shadow promotion gate: completed"
    echo "mode: $PROMOTION_MODE"
    echo "recommendation: $promotion_recommendation"
    echo "note: $promotion_note"
    echo "eligible_for_runtime_enforcement: $eligible_for_runtime_enforcement"
    echo "totals_checked: $total_checked"
    echo "totals_failed: $total_failed"
    echo "trial_passed: $trial_passed"
    echo "trial_failed: $trial_failed"
    echo "trial_gate_failures: $trial_gate_failures"
    echo "trial_missing_report: $trial_missing_report"
    echo "primary_non_shadow_blocker: $primary_non_shadow_blocker"
    echo "declared_shadow_parseable_only: $DECLARED_SHADOW_PARSEABLE_ONLY"
    echo "blocker_breakdown_json: $blocker_breakdown_json"
    echo "report_json: $PROMOTION_REPORT_JSON"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$PROMOTION_MODE" == "1" && "$eligible_for_runtime_enforcement" -ne 1 ]]; then
    echo "error: strict promotion mode requires eligibility for runtime declared-identifier enforcement" >&2
    cat "$PROMOTION_REPORT_JSON" >&2
    exit 1
fi

if [[ "$PROMOTION_MODE" == "auto" && "$trial_missing_report" -eq "$TRIALS" ]]; then
    echo "error: all promotion trials failed before producing declared shadow reports" >&2
    cat "$PROMOTION_REPORT_JSON" >&2
    exit 1
fi

exit 0
