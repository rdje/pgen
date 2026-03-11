#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_STRICT_PROMOTION_STATE_DIR:-$RUST_DIR/target/vhdl_strict_promotion_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_TXT="$STATE_DIR/summary.txt"
PROMOTION_REPORT_JSON="$WORK_DIR/vhdl_strict_promotion_report.json"

PROMOTION_MODE="${PGEN_VHDL_STRICT_PROMOTION_MODE:-auto}" # auto|0|1
TRIALS="${PGEN_VHDL_STRICT_PROMOTION_TRIALS:-3}"
SAMPLE_COUNT="${PGEN_VHDL_STRICT_PROMOTION_COUNT:-8}"
SEED_BASE="${PGEN_VHDL_STRICT_PROMOTION_SEED_BASE:-22001}"
SEED_STRIDE="${PGEN_VHDL_STRICT_PROMOTION_SEED_STRIDE:-250000}"
PARSE_FULL_MODE="${PGEN_VHDL_STRICT_PROMOTION_PARSE_FULL_MODE:-auto}" # auto|0|1
REALISTIC_CORPUS_MODE="${PGEN_VHDL_STRICT_PROMOTION_REALISTIC_CORPUS_MODE:-auto}" # auto|0|1
TARGET_MIN_RATIO="${PGEN_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO:-0}" # integer 0..100
REQUIRE_REALISTIC_PARITY="${PGEN_VHDL_STRICT_PROMOTION_REQUIRE_REALISTIC_PARITY:-1}" # 0|1

if [[ "$PROMOTION_MODE" != "auto" && "$PROMOTION_MODE" != "0" && "$PROMOTION_MODE" != "1" ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$TRIALS" =~ ^[0-9]+$ ]] || [[ "$TRIALS" -lt 1 ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_TRIALS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_SEED_BASE must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$SEED_STRIDE" =~ ^[0-9]+$ ]] || [[ "$SEED_STRIDE" -lt 1 ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_SEED_STRIDE must be an integer >= 1" >&2
    exit 2
fi
if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$REALISTIC_CORPUS_MODE" != "auto" && "$REALISTIC_CORPUS_MODE" != "0" && "$REALISTIC_CORPUS_MODE" != "1" ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_REALISTIC_CORPUS_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$TARGET_MIN_RATIO" =~ ^[0-9]+$ ]] || [[ "$TARGET_MIN_RATIO" -lt 0 ]] || [[ "$TARGET_MIN_RATIO" -gt 100 ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO must be an integer between 0 and 100" >&2
    exit 2
fi
if ! [[ "$REQUIRE_REALISTIC_PARITY" =~ ^[01]$ ]]; then
    echo "error: PGEN_VHDL_STRICT_PROMOTION_REQUIRE_REALISTIC_PARITY must be 0 or 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

echo "==> VHDL strict promotion gate"
echo "state_dir: $STATE_DIR"
echo "promotion_mode: $PROMOTION_MODE"
echo "trials: $TRIALS"
echo "sample_count: $SAMPLE_COUNT"
echo "seed_base: $SEED_BASE"
echo "seed_stride: $SEED_STRIDE"
echo "parse_full_mode: $PARSE_FULL_MODE"
echo "realistic_corpus_mode: $REALISTIC_CORPUS_MODE"
echo "target_min_ratio: $TARGET_MIN_RATIO"
echo "require_realistic_parity: $REQUIRE_REALISTIC_PARITY"

if [[ "$PROMOTION_MODE" == "0" ]]; then
    jq -n \
        --arg mode "$PROMOTION_MODE" \
        --arg note "promotion gate disabled by mode" \
        '{
            mode: $mode,
            status: "skipped",
            note: $note,
            eligibility: { eligible_for_required_strict_mode: false },
            parseability_generation: {
                observed: {
                    requested_total: 0,
                    attempts_total: 0,
                    accepted_total: 0,
                    rejected_total: 0,
                    parser_rejections_total: 0,
                    generation_errors_total: 0,
                    empty_generations_total: 0,
                    acceptance_rate_percent: 0
                }
            },
            closed_loop_parseability_shadow: {
                observed: {
                    requested_total: 0,
                    attempts_total: 0,
                    accepted_total: 0,
                    rejected_total: 0,
                    parser_rejections_total: 0,
                    generation_errors_total: 0,
                    empty_generations_total: 0,
                    acceptance_rate_percent: 0
                },
                target_drive_validation: {
                    alternate_entry_attempts_total: 0,
                    alternate_entry_accepted_outputs_total: 0,
                    alternate_entry_rejected_outputs_total: 0
                }
            }
        }' >"$PROMOTION_REPORT_JSON"
    {
        echo "VHDL strict promotion gate: skipped"
        echo "mode: $PROMOTION_MODE"
        echo "note: promotion gate disabled by mode"
        echo "report_json: $PROMOTION_REPORT_JSON"
    } >"$SUMMARY_TXT"
    cat "$SUMMARY_TXT"
    exit 0
fi

extract_trial_ratio() {
    local summary_txt="$1"
    local trial_log="$2"
    local row=""
    row="$(sed -nE 's/^parse_full_passes: ([0-9]+)\/([0-9]+)$/\1 \2/p' "$summary_txt" | tail -n 1 || true)"
    if [[ -z "$row" ]]; then
        row="$(sed -nE 's/^parse_full_passes: ([0-9]+)\/([0-9]+)$/\1 \2/p' "$trial_log" | tail -n 1 || true)"
    fi
    if [[ -z "$row" ]]; then
        return 1
    fi
    local pass_count total_count
    pass_count="$(awk '{print $1}' <<<"$row")"
    total_count="$(awk '{print $2}' <<<"$row")"
    if [[ -z "$total_count" || "$total_count" -eq 0 ]]; then
        return 1
    fi
    echo $(( (pass_count * 100) / total_count ))
    return 0
}

TRIAL_CASES_JSONL="$WORK_DIR/trial_cases.jsonl"
: >"$TRIAL_CASES_JSONL"

trial_passed=0
trial_ratio_failures=0
trial_parity_failures=0
trial_gate_failures=0
trial_missing_ratio=0
ratio_count=0
ratio_sum=0
ratio_min=101
ratio_max=0
parseability_generation_requested_total=0
parseability_generation_attempts_total=0
parseability_generation_accepted_total=0
parseability_generation_rejected_total=0
parseability_generation_parser_rejections_total=0
parseability_generation_generation_errors_total=0
parseability_generation_empty_generations_total=0
closed_loop_parseability_shadow_requested_total=0
closed_loop_parseability_shadow_attempts_total=0
closed_loop_parseability_shadow_accepted_total=0
closed_loop_parseability_shadow_rejected_total=0
closed_loop_parseability_shadow_parser_rejections_total=0
closed_loop_parseability_shadow_generation_errors_total=0
closed_loop_parseability_shadow_empty_generations_total=0
closed_loop_parseability_shadow_alternate_entry_attempts_total=0
closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total=0
closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total=0

summary_value_from_file() {
    local key="$1"
    local summary_file="$2"
    if [[ ! -f "$summary_file" ]]; then
        return 0
    fi
    awk -F': ' -v key="$key" '$1 == key { print substr($0, index($0, $2)) }' "$summary_file" | tail -n 1 || true
}

u64_or_zero() {
    local value="${1:-}"
    if [[ "$value" =~ ^[0-9]+$ ]]; then
        printf '%s\n' "$value"
    else
        printf '0\n'
    fi
}

number_or_zero() {
    local value="${1:-}"
    if [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
        printf '%s\n' "$value"
    else
        printf '0\n'
    fi
}

rate_from_counts() {
    local accepted="${1:-0}"
    local attempts="${2:-0}"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' \
        "$accepted" \
        "$attempts"
}

normalize_boolish_or_unknown() {
    local value="${1:-}"
    case "$value" in
        1|true|TRUE|True)
            printf 'true\n'
            ;;
        0|false|FALSE|False)
            printf 'false\n'
            ;;
        *)
            printf 'unknown\n'
            ;;
    esac
}

for ((trial_idx = 0; trial_idx < TRIALS; trial_idx++)); do
    trial_seed_base=$((SEED_BASE + (trial_idx * SEED_STRIDE)))
    trial_state_dir="$WORK_DIR/trial_${trial_idx}"
    trial_log="$LOG_DIR/trial_${trial_idx}.log"
    trial_summary_txt="$trial_state_dir/summary.txt"
    trial_report_json="$trial_state_dir/work/vhdl_realistic_corpus_report.json"
    trial_parseability_generation_report_json="$trial_state_dir/work/vhdl_parseability_generation_report.json"
    trial_closed_loop_parseability_shadow_report_json="$trial_state_dir/work/closed_loop_replay_parseability_shadow_report.json"
    mkdir -p "$trial_state_dir"

    echo "==> strict_trial_${trial_idx}"
    trial_exit=0
    if (
        cd "$RUST_DIR"
        PGEN_VHDL_STIMULI_QUALITY_STATE_DIR="$trial_state_dir" \
            PGEN_VHDL_STIMULI_QUALITY_COUNT="$SAMPLE_COUNT" \
            PGEN_VHDL_STIMULI_QUALITY_SEED_BASE="$trial_seed_base" \
            PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE="$PARSE_FULL_MODE" \
            PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE="$REALISTIC_CORPUS_MODE" \
            ./scripts/vhdl_stimuli_quality_gate.sh
    ) >"$trial_log" 2>&1; then
        echo "    ok (${trial_log})"
    else
        trial_exit=$?
        echo "    fail (${trial_log})"
    fi

    trial_ratio=""
    if trial_ratio="$(extract_trial_ratio "$trial_summary_txt" "$trial_log")"; then
        ratio_count=$((ratio_count + 1))
        ratio_sum=$((ratio_sum + trial_ratio))
        if (( trial_ratio < ratio_min )); then
            ratio_min="$trial_ratio"
        fi
        if (( trial_ratio > ratio_max )); then
            ratio_max="$trial_ratio"
        fi
    else
        trial_missing_ratio=$((trial_missing_ratio + 1))
    fi

    if [[ ! -f "$trial_parseability_generation_report_json" ]]; then
        trial_parseability_generation_report_json="$(summary_value_from_file "parseability_generation_report_json" "$trial_summary_txt")"
    fi
    if [[ ! -f "$trial_closed_loop_parseability_shadow_report_json" ]]; then
        trial_closed_loop_parseability_shadow_report_json="$(summary_value_from_file "closed_loop_parseability_shadow_report_json" "$trial_summary_txt")"
    fi

    trial_parseability_generation_enabled="unknown"
    trial_parseability_generation_requested_total=0
    trial_parseability_generation_attempts_total=0
    trial_parseability_generation_accepted_total=0
    trial_parseability_generation_rejected_total=0
    trial_parseability_generation_parser_rejections_total=0
    trial_parseability_generation_generation_errors_total=0
    trial_parseability_generation_empty_generations_total=0
    trial_parseability_generation_acceptance_rate_percent="0.00"

    if [[ -f "$trial_parseability_generation_report_json" ]]; then
        trial_parseability_generation_enabled="$(jq -er 'if has("enabled") then (if .enabled then "true" else "false" end) else "unknown" end' "$trial_parseability_generation_report_json" 2>/dev/null || echo "unknown")"
        trial_parseability_generation_requested_total="$(jq -er '(.totals.requested_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_attempts_total="$(jq -er '(.totals.attempts_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_accepted_total="$(jq -er '(.totals.accepted_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_rejected_total="$(jq -er '(.totals.rejected_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_parser_rejections_total="$(jq -er '(.totals.parser_rejections_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_generation_errors_total="$(jq -er '(.totals.generation_errors_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_empty_generations_total="$(jq -er '(.totals.empty_generations_total // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo 0)"
        trial_parseability_generation_acceptance_rate_percent="$(jq -er '(.totals.acceptance_rate_percent // 0) | numbers' "$trial_parseability_generation_report_json" 2>/dev/null || echo "0.00")"
    elif [[ -f "$trial_summary_txt" ]]; then
        trial_parseability_generation_enabled="$(summary_value_from_file "parseability_generation_enabled" "$trial_summary_txt")"
        trial_parseability_generation_requested_total="$(summary_value_from_file "parseability_generation_requested_total" "$trial_summary_txt")"
        trial_parseability_generation_attempts_total="$(summary_value_from_file "parseability_generation_attempts_total" "$trial_summary_txt")"
        trial_parseability_generation_accepted_total="$(summary_value_from_file "parseability_generation_accepted_total" "$trial_summary_txt")"
        trial_parseability_generation_rejected_total="$(summary_value_from_file "parseability_generation_rejected_total" "$trial_summary_txt")"
        trial_parseability_generation_parser_rejections_total="$(summary_value_from_file "parseability_generation_parser_rejections_total" "$trial_summary_txt")"
        trial_parseability_generation_generation_errors_total="$(summary_value_from_file "parseability_generation_generation_errors_total" "$trial_summary_txt")"
        trial_parseability_generation_empty_generations_total="$(summary_value_from_file "parseability_generation_empty_generations_total" "$trial_summary_txt")"
        trial_parseability_generation_acceptance_rate_percent="$(summary_value_from_file "parseability_generation_acceptance_rate_percent" "$trial_summary_txt")"
    fi

    trial_closed_loop_parseability_shadow_enabled="unknown"
    trial_closed_loop_parseability_shadow_effective="unknown"
    trial_closed_loop_parseability_shadow_requested_total=0
    trial_closed_loop_parseability_shadow_attempts_total=0
    trial_closed_loop_parseability_shadow_accepted_total=0
    trial_closed_loop_parseability_shadow_rejected_total=0
    trial_closed_loop_parseability_shadow_parser_rejections_total=0
    trial_closed_loop_parseability_shadow_generation_errors_total=0
    trial_closed_loop_parseability_shadow_empty_generations_total=0
    trial_closed_loop_parseability_shadow_alternate_entry_attempts_total=0
    trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total=0
    trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total=0
    trial_closed_loop_parseability_shadow_acceptance_rate_percent="0.00"

    if [[ -f "$trial_closed_loop_parseability_shadow_report_json" ]]; then
        trial_closed_loop_parseability_shadow_enabled="$(jq -er 'if has("enabled") then (if .enabled then "true" else "false" end) else "unknown" end' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo "unknown")"
        trial_closed_loop_parseability_shadow_effective="$(jq -er '.effective_mode // "unknown"' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo "unknown")"
        trial_closed_loop_parseability_shadow_requested_total="$(jq -er '(.summary.requested // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_attempts_total="$(jq -er '(.summary.attempts // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_accepted_total="$(jq -er '(.summary.accepted // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_rejected_total="$(jq -er '(.summary.rejected // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_parser_rejections_total="$(jq -er '(.summary.parser_rejections // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_generation_errors_total="$(jq -er '(.summary.generation_errors // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_empty_generations_total="$(jq -er '(.summary.empty_generations // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_alternate_entry_attempts_total="$(jq -er '(.target_drive_validation.alternate_entry_attempts_total // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total="$(jq -er '(.target_drive_validation.alternate_entry_accepted_outputs_total // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total="$(jq -er '(.target_drive_validation.alternate_entry_rejected_outputs_total // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo 0)"
        trial_closed_loop_parseability_shadow_acceptance_rate_percent="$(jq -er '(.summary.acceptance_rate_percent // 0) | numbers' "$trial_closed_loop_parseability_shadow_report_json" 2>/dev/null || echo "0.00")"

        if [[ "$trial_closed_loop_parseability_shadow_enabled" == "unknown" && -f "$trial_summary_txt" ]]; then
            trial_closed_loop_parseability_shadow_enabled="$(summary_value_from_file "closed_loop_parseability_shadow_enabled" "$trial_summary_txt")"
        fi
        if [[ "$trial_closed_loop_parseability_shadow_effective" == "unknown" && -f "$trial_summary_txt" ]]; then
            trial_closed_loop_parseability_shadow_effective="$(summary_value_from_file "closed_loop_parseability_shadow_effective" "$trial_summary_txt")"
        fi
        if [[ "$trial_closed_loop_parseability_shadow_acceptance_rate_percent" == "0.00" && -f "$trial_summary_txt" ]]; then
            summary_shadow_acceptance_rate_percent="$(summary_value_from_file "closed_loop_parseability_shadow_acceptance_rate_percent" "$trial_summary_txt")"
            if [[ -n "$summary_shadow_acceptance_rate_percent" ]]; then
                trial_closed_loop_parseability_shadow_acceptance_rate_percent="$summary_shadow_acceptance_rate_percent"
            fi
        fi
    elif [[ -f "$trial_summary_txt" ]]; then
        trial_closed_loop_parseability_shadow_enabled="$(summary_value_from_file "closed_loop_parseability_shadow_enabled" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_effective="$(summary_value_from_file "closed_loop_parseability_shadow_effective" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_requested_total="$(summary_value_from_file "closed_loop_parseability_shadow_requested_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_attempts_total="$(summary_value_from_file "closed_loop_parseability_shadow_attempts_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_accepted_total="$(summary_value_from_file "closed_loop_parseability_shadow_accepted_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_rejected_total="$(summary_value_from_file "closed_loop_parseability_shadow_rejected_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_parser_rejections_total="$(summary_value_from_file "closed_loop_parseability_shadow_parser_rejections_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_generation_errors_total="$(summary_value_from_file "closed_loop_parseability_shadow_generation_errors_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_empty_generations_total="$(summary_value_from_file "closed_loop_parseability_shadow_empty_generations_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_alternate_entry_attempts_total="$(summary_value_from_file "closed_loop_parseability_shadow_alternate_entry_attempts_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total="$(summary_value_from_file "closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total="$(summary_value_from_file "closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total" "$trial_summary_txt")"
        trial_closed_loop_parseability_shadow_acceptance_rate_percent="$(summary_value_from_file "closed_loop_parseability_shadow_acceptance_rate_percent" "$trial_summary_txt")"
    fi

    trial_parseability_generation_requested_total="$(u64_or_zero "$trial_parseability_generation_requested_total")"
    trial_parseability_generation_attempts_total="$(u64_or_zero "$trial_parseability_generation_attempts_total")"
    trial_parseability_generation_accepted_total="$(u64_or_zero "$trial_parseability_generation_accepted_total")"
    trial_parseability_generation_rejected_total="$(u64_or_zero "$trial_parseability_generation_rejected_total")"
    trial_parseability_generation_parser_rejections_total="$(u64_or_zero "$trial_parseability_generation_parser_rejections_total")"
    trial_parseability_generation_generation_errors_total="$(u64_or_zero "$trial_parseability_generation_generation_errors_total")"
    trial_parseability_generation_empty_generations_total="$(u64_or_zero "$trial_parseability_generation_empty_generations_total")"
    trial_parseability_generation_acceptance_rate_percent="$(number_or_zero "$trial_parseability_generation_acceptance_rate_percent")"
    trial_parseability_generation_enabled="$(normalize_boolish_or_unknown "$trial_parseability_generation_enabled")"
    trial_closed_loop_parseability_shadow_requested_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_requested_total")"
    trial_closed_loop_parseability_shadow_attempts_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_attempts_total")"
    trial_closed_loop_parseability_shadow_accepted_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_accepted_total")"
    trial_closed_loop_parseability_shadow_rejected_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_rejected_total")"
    trial_closed_loop_parseability_shadow_parser_rejections_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_parser_rejections_total")"
    trial_closed_loop_parseability_shadow_generation_errors_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_generation_errors_total")"
    trial_closed_loop_parseability_shadow_empty_generations_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_empty_generations_total")"
    trial_closed_loop_parseability_shadow_alternate_entry_attempts_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_alternate_entry_attempts_total")"
    trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total")"
    trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total="$(u64_or_zero "$trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total")"
    trial_closed_loop_parseability_shadow_acceptance_rate_percent="$(rate_from_counts "$trial_closed_loop_parseability_shadow_accepted_total" "$trial_closed_loop_parseability_shadow_attempts_total")"
    trial_closed_loop_parseability_shadow_enabled="$(normalize_boolish_or_unknown "$trial_closed_loop_parseability_shadow_enabled")"

    parseability_generation_requested_total=$((parseability_generation_requested_total + trial_parseability_generation_requested_total))
    parseability_generation_attempts_total=$((parseability_generation_attempts_total + trial_parseability_generation_attempts_total))
    parseability_generation_accepted_total=$((parseability_generation_accepted_total + trial_parseability_generation_accepted_total))
    parseability_generation_rejected_total=$((parseability_generation_rejected_total + trial_parseability_generation_rejected_total))
    parseability_generation_parser_rejections_total=$((parseability_generation_parser_rejections_total + trial_parseability_generation_parser_rejections_total))
    parseability_generation_generation_errors_total=$((parseability_generation_generation_errors_total + trial_parseability_generation_generation_errors_total))
    parseability_generation_empty_generations_total=$((parseability_generation_empty_generations_total + trial_parseability_generation_empty_generations_total))
    closed_loop_parseability_shadow_requested_total=$((closed_loop_parseability_shadow_requested_total + trial_closed_loop_parseability_shadow_requested_total))
    closed_loop_parseability_shadow_attempts_total=$((closed_loop_parseability_shadow_attempts_total + trial_closed_loop_parseability_shadow_attempts_total))
    closed_loop_parseability_shadow_accepted_total=$((closed_loop_parseability_shadow_accepted_total + trial_closed_loop_parseability_shadow_accepted_total))
    closed_loop_parseability_shadow_rejected_total=$((closed_loop_parseability_shadow_rejected_total + trial_closed_loop_parseability_shadow_rejected_total))
    closed_loop_parseability_shadow_parser_rejections_total=$((closed_loop_parseability_shadow_parser_rejections_total + trial_closed_loop_parseability_shadow_parser_rejections_total))
    closed_loop_parseability_shadow_generation_errors_total=$((closed_loop_parseability_shadow_generation_errors_total + trial_closed_loop_parseability_shadow_generation_errors_total))
    closed_loop_parseability_shadow_empty_generations_total=$((closed_loop_parseability_shadow_empty_generations_total + trial_closed_loop_parseability_shadow_empty_generations_total))
    closed_loop_parseability_shadow_alternate_entry_attempts_total=$((closed_loop_parseability_shadow_alternate_entry_attempts_total + trial_closed_loop_parseability_shadow_alternate_entry_attempts_total))
    closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total=$((closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total + trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total))
    closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total=$((closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total + trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total))

    realistic_enabled=0
    realistic_expected_pass=""
    realistic_expected_fail=""
    realistic_observed_pass=""
    realistic_observed_fail=""
    realistic_parity_ok=0
    if [[ -f "$trial_report_json" ]]; then
        realistic_enabled="$(jq -er 'if (.enabled // false) then 1 else 0 end' "$trial_report_json" 2>/dev/null || echo 0)"
        realistic_expected_pass="$(jq -er '.totals.expected_pass_total // -1' "$trial_report_json" 2>/dev/null || echo -1)"
        realistic_expected_fail="$(jq -er '.totals.expected_fail_total // -1' "$trial_report_json" 2>/dev/null || echo -1)"
        realistic_observed_pass="$(jq -er '.totals.observed_parse_pass_total // -1' "$trial_report_json" 2>/dev/null || echo -1)"
        realistic_observed_fail="$(jq -er '.totals.observed_parse_fail_total // -1' "$trial_report_json" 2>/dev/null || echo -1)"
        if [[ "$realistic_expected_pass" == "$realistic_observed_pass" && "$realistic_expected_fail" == "$realistic_observed_fail" ]]; then
            realistic_parity_ok=1
        fi
    fi

    trial_status="gate_fail"
    trial_note="vhdl stimuli quality gate failed"
    trial_blocker_key="vhdl_quality_gate_failed"
    trial_blocker_detail=""

    if [[ "$trial_exit" -eq 0 ]]; then
        if [[ -z "$trial_ratio" ]]; then
            trial_status="gate_fail"
            trial_note="trial completed but parse_full ratio telemetry was unavailable"
            trial_blocker_key="missing_ratio_telemetry"
            trial_gate_failures=$((trial_gate_failures + 1))
        elif [[ "$trial_ratio" -lt "$TARGET_MIN_RATIO" ]]; then
            trial_status="ratio_fail"
            trial_note="parse_full ratio below target (${trial_ratio}% < ${TARGET_MIN_RATIO}%)"
            trial_blocker_key="parse_full_ratio_threshold_not_met"
            trial_blocker_detail="observed_ratio=${trial_ratio}% target_min_ratio=${TARGET_MIN_RATIO}%"
            trial_ratio_failures=$((trial_ratio_failures + 1))
        elif [[ "$REQUIRE_REALISTIC_PARITY" -eq 1 && ! -f "$trial_report_json" ]]; then
            trial_status="parity_fail"
            trial_note="realistic corpus report missing while parity is required"
            trial_blocker_key="missing_realistic_report"
            trial_parity_failures=$((trial_parity_failures + 1))
        elif [[ "$REQUIRE_REALISTIC_PARITY" -eq 1 && "$realistic_enabled" -ne 1 ]]; then
            trial_status="parity_fail"
            trial_note="realistic corpus stage was not enabled while parity is required"
            trial_blocker_key="realistic_corpus_disabled"
            trial_parity_failures=$((trial_parity_failures + 1))
        elif [[ "$REQUIRE_REALISTIC_PARITY" -eq 1 && "$realistic_parity_ok" -ne 1 ]]; then
            trial_status="parity_fail"
            trial_note="realistic corpus observed pass/fail totals diverged from expected totals"
            trial_blocker_key="realistic_corpus_parity_mismatch"
            trial_blocker_detail="expected_pass=${realistic_expected_pass} observed_pass=${realistic_observed_pass} expected_fail=${realistic_expected_fail} observed_fail=${realistic_observed_fail}"
            trial_parity_failures=$((trial_parity_failures + 1))
        else
            trial_status="pass"
            trial_note="strict promotion trial passed"
            trial_blocker_key="none"
            trial_passed=$((trial_passed + 1))
        fi
    else
        trial_gate_failures=$((trial_gate_failures + 1))
    fi

    if [[ -n "$trial_ratio" ]]; then
        trial_note="${trial_note}; observed_ratio=${trial_ratio}%"
    fi

    jq -n \
        --argjson trial_index "$trial_idx" \
        --argjson seed_base "$trial_seed_base" \
        --arg log_file "$trial_log" \
        --arg summary_file "$trial_summary_txt" \
        --arg report_json "$trial_report_json" \
        --argjson exit_code "$trial_exit" \
        --arg status "$trial_status" \
        --arg note "$trial_note" \
        --arg blocker_key "$trial_blocker_key" \
        --arg blocker_detail "$trial_blocker_detail" \
        --argjson observed_ratio "$(if [[ -n "$trial_ratio" ]]; then echo "$trial_ratio"; else echo "null"; fi)" \
        --argjson realistic_enabled "$realistic_enabled" \
        --argjson realistic_parity_ok "$realistic_parity_ok" \
        --arg parseability_generation_enabled "$trial_parseability_generation_enabled" \
        --arg parseability_generation_report_json "$trial_parseability_generation_report_json" \
        --arg closed_loop_parseability_shadow_enabled "$trial_closed_loop_parseability_shadow_enabled" \
        --arg closed_loop_parseability_shadow_effective "$trial_closed_loop_parseability_shadow_effective" \
        --arg closed_loop_parseability_shadow_report_json "$trial_closed_loop_parseability_shadow_report_json" \
        --argjson realistic_expected_pass "$(if [[ -n "$realistic_expected_pass" ]]; then echo "$realistic_expected_pass"; else echo "null"; fi)" \
        --argjson realistic_expected_fail "$(if [[ -n "$realistic_expected_fail" ]]; then echo "$realistic_expected_fail"; else echo "null"; fi)" \
        --argjson realistic_observed_pass "$(if [[ -n "$realistic_observed_pass" ]]; then echo "$realistic_observed_pass"; else echo "null"; fi)" \
        --argjson realistic_observed_fail "$(if [[ -n "$realistic_observed_fail" ]]; then echo "$realistic_observed_fail"; else echo "null"; fi)" \
        --argjson parseability_generation_requested_total "$trial_parseability_generation_requested_total" \
        --argjson parseability_generation_attempts_total "$trial_parseability_generation_attempts_total" \
        --argjson parseability_generation_accepted_total "$trial_parseability_generation_accepted_total" \
        --argjson parseability_generation_rejected_total "$trial_parseability_generation_rejected_total" \
        --argjson parseability_generation_parser_rejections_total "$trial_parseability_generation_parser_rejections_total" \
        --argjson parseability_generation_generation_errors_total "$trial_parseability_generation_generation_errors_total" \
        --argjson parseability_generation_empty_generations_total "$trial_parseability_generation_empty_generations_total" \
        --argjson parseability_generation_acceptance_rate_percent "$trial_parseability_generation_acceptance_rate_percent" \
        --argjson closed_loop_parseability_shadow_requested_total "$trial_closed_loop_parseability_shadow_requested_total" \
        --argjson closed_loop_parseability_shadow_attempts_total "$trial_closed_loop_parseability_shadow_attempts_total" \
        --argjson closed_loop_parseability_shadow_accepted_total "$trial_closed_loop_parseability_shadow_accepted_total" \
        --argjson closed_loop_parseability_shadow_rejected_total "$trial_closed_loop_parseability_shadow_rejected_total" \
        --argjson closed_loop_parseability_shadow_parser_rejections_total "$trial_closed_loop_parseability_shadow_parser_rejections_total" \
        --argjson closed_loop_parseability_shadow_generation_errors_total "$trial_closed_loop_parseability_shadow_generation_errors_total" \
        --argjson closed_loop_parseability_shadow_empty_generations_total "$trial_closed_loop_parseability_shadow_empty_generations_total" \
        --argjson closed_loop_parseability_shadow_alternate_entry_attempts_total "$trial_closed_loop_parseability_shadow_alternate_entry_attempts_total" \
        --argjson closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total "$trial_closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total" \
        --argjson closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total "$trial_closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total" \
        --argjson closed_loop_parseability_shadow_acceptance_rate_percent "$trial_closed_loop_parseability_shadow_acceptance_rate_percent" \
        '{
            trial_index: $trial_index,
            seed_base: $seed_base,
            log_file: $log_file,
            summary_file: $summary_file,
            realistic_report_json: $report_json,
            exit_code: $exit_code,
            status: $status,
            note: $note,
            blocker_key: $blocker_key,
            blocker_detail: $blocker_detail,
            observed_ratio_percent: $observed_ratio,
            realistic_corpus: {
                enabled: ($realistic_enabled == 1),
                parity_ok: ($realistic_parity_ok == 1),
                expected_pass_total: $realistic_expected_pass,
                expected_fail_total: $realistic_expected_fail,
                observed_pass_total: $realistic_observed_pass,
                observed_fail_total: $realistic_observed_fail
            },
            parseability_generation: {
                enabled: $parseability_generation_enabled,
                report_json: $parseability_generation_report_json,
                observed: {
                    requested_total: $parseability_generation_requested_total,
                    attempts_total: $parseability_generation_attempts_total,
                    accepted_total: $parseability_generation_accepted_total,
                    rejected_total: $parseability_generation_rejected_total,
                    parser_rejections_total: $parseability_generation_parser_rejections_total,
                    generation_errors_total: $parseability_generation_generation_errors_total,
                    empty_generations_total: $parseability_generation_empty_generations_total,
                    acceptance_rate_percent: $parseability_generation_acceptance_rate_percent
                }
            },
            closed_loop_parseability_shadow: {
                enabled: $closed_loop_parseability_shadow_enabled,
                effective_mode: $closed_loop_parseability_shadow_effective,
                report_json: $closed_loop_parseability_shadow_report_json,
                observed: {
                    requested_total: $closed_loop_parseability_shadow_requested_total,
                    attempts_total: $closed_loop_parseability_shadow_attempts_total,
                    accepted_total: $closed_loop_parseability_shadow_accepted_total,
                    rejected_total: $closed_loop_parseability_shadow_rejected_total,
                    parser_rejections_total: $closed_loop_parseability_shadow_parser_rejections_total,
                    generation_errors_total: $closed_loop_parseability_shadow_generation_errors_total,
                    empty_generations_total: $closed_loop_parseability_shadow_empty_generations_total,
                    acceptance_rate_percent: $closed_loop_parseability_shadow_acceptance_rate_percent
                },
                target_drive_validation: {
                    alternate_entry_attempts_total: $closed_loop_parseability_shadow_alternate_entry_attempts_total,
                    alternate_entry_accepted_outputs_total: $closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total,
                    alternate_entry_rejected_outputs_total: $closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total
                }
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
primary_blocker="$(jq -r '
    if length == 0 then
        "none"
    else
        max_by(.count).key
    end
' <<<"$blocker_breakdown_json")"

observed_ratio_avg=0
if (( ratio_count > 0 )); then
    observed_ratio_avg=$((ratio_sum / ratio_count))
else
    ratio_min=0
    ratio_max=0
fi

eligible_for_required_strict_mode=0
promotion_recommendation="hold"
promotion_note=""
if [[ "$trial_gate_failures" -gt 0 ]]; then
    promotion_note="one or more trials failed the underlying vhdl_stimuli_quality_gate"
elif [[ "$trial_ratio_failures" -gt 0 ]]; then
    promotion_note="one or more trials did not meet the parse_full ratio threshold"
elif [[ "$trial_parity_failures" -gt 0 ]]; then
    promotion_note="one or more trials failed realistic corpus parity checks"
elif [[ "$trial_missing_ratio" -gt 0 ]]; then
    promotion_note="one or more trials did not expose parse_full ratio telemetry"
elif [[ "$trial_passed" -eq "$TRIALS" ]]; then
    eligible_for_required_strict_mode=1
    promotion_recommendation="enable_required_strict_mode"
    promotion_note="all deterministic VHDL strict-promotion trials passed"
else
    promotion_note="insufficient strict-promotion evidence"
fi

parseability_generation_acceptance_rate_percent="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_generation_accepted_total" "$parseability_generation_attempts_total")"
closed_loop_parseability_shadow_acceptance_rate_percent="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$closed_loop_parseability_shadow_accepted_total" "$closed_loop_parseability_shadow_attempts_total")"

jq -n \
    --arg mode "$PROMOTION_MODE" \
    --arg recommendation "$promotion_recommendation" \
    --arg note "$promotion_note" \
    --arg parse_full_mode "$PARSE_FULL_MODE" \
    --arg realistic_corpus_mode "$REALISTIC_CORPUS_MODE" \
    --argjson target_min_ratio "$TARGET_MIN_RATIO" \
    --argjson require_realistic_parity "$REQUIRE_REALISTIC_PARITY" \
    --argjson trials "$TRIALS" \
    --argjson sample_count "$SAMPLE_COUNT" \
    --argjson seed_base "$SEED_BASE" \
    --argjson seed_stride "$SEED_STRIDE" \
    --argjson eligible "$eligible_for_required_strict_mode" \
    --argjson trial_passed "$trial_passed" \
    --argjson trial_ratio_failures "$trial_ratio_failures" \
    --argjson trial_parity_failures "$trial_parity_failures" \
    --argjson trial_gate_failures "$trial_gate_failures" \
    --argjson trial_missing_ratio "$trial_missing_ratio" \
    --argjson observed_ratio_min "$ratio_min" \
    --argjson observed_ratio_max "$ratio_max" \
    --argjson observed_ratio_avg "$observed_ratio_avg" \
    --arg primary_blocker "$primary_blocker" \
    --argjson blocker_breakdown "$blocker_breakdown_json" \
    --argjson parseability_generation_requested_total "$parseability_generation_requested_total" \
    --argjson parseability_generation_attempts_total "$parseability_generation_attempts_total" \
    --argjson parseability_generation_accepted_total "$parseability_generation_accepted_total" \
    --argjson parseability_generation_rejected_total "$parseability_generation_rejected_total" \
    --argjson parseability_generation_parser_rejections_total "$parseability_generation_parser_rejections_total" \
    --argjson parseability_generation_generation_errors_total "$parseability_generation_generation_errors_total" \
    --argjson parseability_generation_empty_generations_total "$parseability_generation_empty_generations_total" \
    --argjson parseability_generation_acceptance_rate_percent "$parseability_generation_acceptance_rate_percent" \
    --argjson closed_loop_parseability_shadow_requested_total "$closed_loop_parseability_shadow_requested_total" \
    --argjson closed_loop_parseability_shadow_attempts_total "$closed_loop_parseability_shadow_attempts_total" \
    --argjson closed_loop_parseability_shadow_accepted_total "$closed_loop_parseability_shadow_accepted_total" \
    --argjson closed_loop_parseability_shadow_rejected_total "$closed_loop_parseability_shadow_rejected_total" \
    --argjson closed_loop_parseability_shadow_parser_rejections_total "$closed_loop_parseability_shadow_parser_rejections_total" \
    --argjson closed_loop_parseability_shadow_generation_errors_total "$closed_loop_parseability_shadow_generation_errors_total" \
    --argjson closed_loop_parseability_shadow_empty_generations_total "$closed_loop_parseability_shadow_empty_generations_total" \
    --argjson closed_loop_parseability_shadow_alternate_entry_attempts_total "$closed_loop_parseability_shadow_alternate_entry_attempts_total" \
    --argjson closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total "$closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total" \
    --argjson closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total "$closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total" \
    --argjson closed_loop_parseability_shadow_acceptance_rate_percent "$closed_loop_parseability_shadow_acceptance_rate_percent" \
    --argjson trials_json "$trial_cases_json" \
    '{
        mode: $mode,
        status: "completed",
        recommendation: $recommendation,
        note: $note,
        parse_full_mode: $parse_full_mode,
        realistic_corpus_mode: $realistic_corpus_mode,
        target_min_ratio: $target_min_ratio,
        require_realistic_parity: ($require_realistic_parity == 1),
        eligibility: {
            eligible_for_required_strict_mode: ($eligible == 1)
        },
        totals: {
            trials: $trials,
            sample_count_per_trial: $sample_count,
            seed_base: $seed_base,
            seed_stride: $seed_stride,
            trial_passed: $trial_passed,
            trial_ratio_failures: $trial_ratio_failures,
            trial_parity_failures: $trial_parity_failures,
            trial_gate_failures: $trial_gate_failures,
            trial_missing_ratio: $trial_missing_ratio,
            observed_ratio_min: $observed_ratio_min,
            observed_ratio_max: $observed_ratio_max,
            observed_ratio_avg: $observed_ratio_avg
        },
        blockers: {
            failed_trial_count: ($trial_ratio_failures + $trial_parity_failures + $trial_gate_failures),
            primary_blocker: $primary_blocker,
            breakdown: $blocker_breakdown
        },
        parseability_generation: {
            observed: {
                requested_total: $parseability_generation_requested_total,
                attempts_total: $parseability_generation_attempts_total,
                accepted_total: $parseability_generation_accepted_total,
                rejected_total: $parseability_generation_rejected_total,
                parser_rejections_total: $parseability_generation_parser_rejections_total,
                generation_errors_total: $parseability_generation_generation_errors_total,
                empty_generations_total: $parseability_generation_empty_generations_total,
                acceptance_rate_percent: $parseability_generation_acceptance_rate_percent
            }
        },
        closed_loop_parseability_shadow: {
            observed: {
                requested_total: $closed_loop_parseability_shadow_requested_total,
                attempts_total: $closed_loop_parseability_shadow_attempts_total,
                accepted_total: $closed_loop_parseability_shadow_accepted_total,
                rejected_total: $closed_loop_parseability_shadow_rejected_total,
                parser_rejections_total: $closed_loop_parseability_shadow_parser_rejections_total,
                generation_errors_total: $closed_loop_parseability_shadow_generation_errors_total,
                empty_generations_total: $closed_loop_parseability_shadow_empty_generations_total,
                acceptance_rate_percent: $closed_loop_parseability_shadow_acceptance_rate_percent
            },
            target_drive_validation: {
                alternate_entry_attempts_total: $closed_loop_parseability_shadow_alternate_entry_attempts_total,
                alternate_entry_accepted_outputs_total: $closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total,
                alternate_entry_rejected_outputs_total: $closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total
            }
        },
        trials: $trials_json
    }' >"$PROMOTION_REPORT_JSON"

{
    echo "VHDL strict promotion gate: completed"
    echo "mode: $PROMOTION_MODE"
    echo "recommendation: $promotion_recommendation"
    echo "note: $promotion_note"
    echo "eligible_for_required_strict_mode: $eligible_for_required_strict_mode"
    echo "target_min_ratio: $TARGET_MIN_RATIO"
    echo "require_realistic_parity: $REQUIRE_REALISTIC_PARITY"
    echo "seed_base: $SEED_BASE"
    echo "seed_stride: $SEED_STRIDE"
    echo "trial_passed: $trial_passed"
    echo "trial_ratio_failures: $trial_ratio_failures"
    echo "trial_parity_failures: $trial_parity_failures"
    echo "trial_gate_failures: $trial_gate_failures"
    echo "trial_missing_ratio: $trial_missing_ratio"
    echo "primary_blocker: $primary_blocker"
    echo "blocker_breakdown_json: $blocker_breakdown_json"
    echo "observed_ratio_min: $ratio_min"
    echo "observed_ratio_max: $ratio_max"
    echo "observed_ratio_avg: $observed_ratio_avg"
    echo "parseability_generation_attempts_total: $parseability_generation_attempts_total"
    echo "parseability_generation_accepted_total: $parseability_generation_accepted_total"
    echo "parseability_generation_rejected_total: $parseability_generation_rejected_total"
    echo "parseability_generation_acceptance_rate_percent: $parseability_generation_acceptance_rate_percent"
    echo "closed_loop_parseability_shadow_attempts_total: $closed_loop_parseability_shadow_attempts_total"
    echo "closed_loop_parseability_shadow_accepted_total: $closed_loop_parseability_shadow_accepted_total"
    echo "closed_loop_parseability_shadow_rejected_total: $closed_loop_parseability_shadow_rejected_total"
    echo "closed_loop_parseability_shadow_acceptance_rate_percent: $closed_loop_parseability_shadow_acceptance_rate_percent"
    echo "closed_loop_parseability_shadow_alternate_entry_attempts_total: $closed_loop_parseability_shadow_alternate_entry_attempts_total"
    echo "closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total: $closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total"
    echo "closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total: $closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total"
    echo "report_json: $PROMOTION_REPORT_JSON"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$PROMOTION_MODE" == "1" && "$eligible_for_required_strict_mode" -ne 1 ]]; then
    echo "error: strict promotion mode requires strict VHDL-promotion eligibility" >&2
    cat "$PROMOTION_REPORT_JSON" >&2
    exit 1
fi

if [[ "$PROMOTION_MODE" == "auto" && "$trial_gate_failures" -eq "$TRIALS" ]]; then
    echo "error: all VHDL strict-promotion trials failed for gate-execution reasons" >&2
    cat "$PROMOTION_REPORT_JSON" >&2
    exit 1
fi

exit 0
