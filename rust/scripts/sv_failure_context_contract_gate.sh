#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_failure_context_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SV_CONTRACT_FILE="${PGEN_SV_FAILURE_CONTEXT_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json}"
SVPP_POLICY_ENV_FILE="${PGEN_SV_FAILURE_CONTEXT_SVPP_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_lightweight_v0.env}"
SV_GATE_SCRIPT="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"
SV_PARSER_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_parser_aggregate_contract_gate.sh"
SVPP_QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
SVPP_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"

EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"
EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR:-}"

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

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | numbers" "$path"
}

extract_json_string() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | strings" "$path"
}

# ⭐⭐⭐ CI-PARITY-GATE-ROT.5 — "THE ZERO MUST BE EARNED".
#
# ⛔ WHAT WAS HERE BEFORE, AND WHY IT WAS THE BUG. Three assertions read
#   `(.by_failure_context_excerpt | length) < 1` → `error: expected at least one … excerpt`
# i.e. they demanded that a counterexample EXIST. Measured 2026-07-28: the generation surface's own
# report says `requested_total: 1, accepted_total: 1, attempts_total: 1, parser_rejections_total: 0,
# acceptance_rate_percent: 100.00` — one sample requested, accepted first try, so there is genuinely
# nothing to excerpt. The one-sample budget is DELIBERATE and documented: the contract
# `systemverilog_failure_context_v0_contract.json` describes itself as *"one-profile, one-sample"*
# with `"sample_count": 1`, and the assertion and that contract landed in the SAME commit
# `74fc5cb6` (2026-03-15). ⇒ **the assertion could only be satisfied if the SV parser REJECTED its
# own generated sample: it passed when the system was broken and failed when it worked.** That made
# `make -C rust sota_exit_gate` — the flagship aggregate and a README Standard Command — RED, and it
# went unnoticed because nothing ran the aggregate.
#
# ⭐ A FOURTH SHAPE FOR THIS FAMILY. The tree had already found a check that *cannot run* and returns
# green, one that *cannot see* and returns green, and one that *nothing invokes*. This is the
# converse the unifying principle needed: **a check must not depend on the thing it watches being
# broken.**
#
# ⛔ DELETING THE ASSERTIONS WAS REJECTED. Their anti-vacuity intent is real: if the triage silently
# stopped producing excerpts, the whole failure-context surface would be dead and nothing else would
# notice — this gate goes on to publish `.sample_previews[0].failure_context_excerpt` as its headline
# evidence. So the intent is KEPT and only the dependence on a defect is dropped.
#
# ⭐ THIS IS A CORRECTION, NOT A RELAXATION, and it is strictly STRONGER than what it replaces. The
# new form fails in two cases the old one could not even see:
#   (1) a VACUOUS run — the surface generated nothing at all (`attempts_total == 0`). The old form
#       would also have failed, but for the wrong reason and with a message pointing at excerpts;
#       worse, a budget change to "many samples, none attempted" is exactly the rot this gate exists
#       to catch and it had no way to say so.
#   (2) rejections happened and NO excerpt was produced — the excerpt machinery genuinely broken.
#       The old form could not distinguish that from a healthy run, because both read `length == 0`.
# It stops failing in exactly one case, the one where the old form was wrong: a healthy run with
# nothing to report.
#
# ⭐ THE NUMBERS ARE NOT RE-DERIVED HERE. They are read from the summary the owning aggregate gate
# already emits, and the report paths come from that gate's own `proof_surfaces` block — so this
# gate can never end up judging a different run than the triage it is reading. Re-deriving them from
# hand-written paths is the duplicated-metadata shape `CI-PARITY-GATE-ROT.1` found rotting twelve
# times over.
#
# assert_failure_context_zero_is_earned <label> <triage_json> <attempts> <rejections> <origin>
#   attempts   — how many samples the surface actually attempted
#   rejections — parser rejections + generation errors on that surface (the events that MUST leave
#                an excerpt behind); pass the sum, the caller knows which fields its surface has
assert_failure_context_zero_is_earned() {
    local label="$1"
    local triage_json="$2"
    local attempts="$3"
    local rejections="$4"
    local origin="$5"

    local excerpt_kinds counterexamples preview_excerpt
    excerpt_kinds="$(extract_json_number "$triage_json" '(.by_failure_context_excerpt | length)')"
    counterexamples="$(extract_json_number "$triage_json" '(.total_counterexamples | numbers)')"

    # (1) THE SURFACE MUST HAVE BEEN EXERCISED. A run that attempted nothing proves nothing, and
    #     "zero counterexamples" from zero attempts is the vacuous green this whole tree exists to
    #     remove. This catches it DIRECTLY, which the excerpt-count assertion never could.
    if [[ "$attempts" -lt 1 ]]; then
        echo "error: ${label} failure-context surface was never exercised: attempts_total=${attempts}" >&2
        echo "       (source: ${origin})" >&2
        echo "       A zero counterexample count means nothing when nothing was attempted. Check the" >&2
        echo "       sample budget in the contract driving this surface." >&2
        exit 1
    fi

    # (2) A ZERO MUST BE CONSISTENT. No counterexamples is acceptable ONLY when nothing failed. If
    #     the surface recorded rejections and produced no excerpt, the excerpt machinery IS broken —
    #     which is the real defect the old assertion was groping for.
    if [[ "$counterexamples" -lt 1 ]]; then
        if [[ "$rejections" -gt 0 ]]; then
            echo "error: ${label} recorded ${rejections} rejection(s)/error(s) but captured NO counterexample" >&2
            echo "       (source: ${origin}; triage: ${triage_json})" >&2
            echo "       The failure-context capture path is broken: failures occurred and left no excerpt." >&2
            exit 1
        fi
        echo "    ${label}: 0 counterexamples, EARNED (attempts=${attempts}, rejections=0)"
        return 0
    fi

    # (3) A PRESENT EXCERPT MUST BE WELL-FORMED. Counterexamples exist ⇒ there must be at least one
    #     distinct context excerpt AND a non-empty preview, because that preview is what this gate
    #     publishes as its headline evidence. An empty string here is a silent evidence hole.
    if [[ "$excerpt_kinds" -lt 1 ]]; then
        echo "error: ${label} captured ${counterexamples} counterexample(s) but no failure-context excerpt" >&2
        echo "       (triage: ${triage_json})" >&2
        exit 1
    fi
    if ! preview_excerpt="$(extract_json_string "$triage_json" '.sample_previews[0].failure_context_excerpt')" ||
        [[ -z "$preview_excerpt" ]]; then
        echo "error: ${label} has counterexamples and excerpts but .sample_previews[0].failure_context_excerpt" >&2
        echo "       is missing or empty (triage: ${triage_json}) — this gate publishes that value as evidence." >&2
        exit 1
    fi
    echo "    ${label}: ${counterexamples} counterexample(s), ${excerpt_kinds} distinct excerpt(s), preview non-empty"
}

require_tool jq
require_file "$SV_CONTRACT_FILE"
require_file "$SVPP_POLICY_ENV_FILE"
require_file "$SV_GATE_SCRIPT"
require_file "$SV_PARSER_AGGREGATE_SCRIPT"
require_file "$SVPP_QUALITY_GATE_SCRIPT"
require_file "$SVPP_AGGREGATE_SCRIPT"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

sv_quality_state_dir="$WORK_DIR/systemverilog_failure_context_quality_state"
if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    sv_quality_state_dir="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR"
else
    run_logged "systemverilog_failure_context_quality_gate" env \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$SV_CONTRACT_FILE" \
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
        "$SV_GATE_SCRIPT"
fi

sv_parser_aggregate_state_dir="$WORK_DIR/sv_parser_aggregate_contract_gate"
if [[ -n "$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR" ]]; then
    sv_parser_aggregate_state_dir="$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR"
else
    run_logged "systemverilog_failure_context_aggregate_contract_gate" env \
        PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR="$sv_parser_aggregate_state_dir" \
        PGEN_SV_PARSER_AGGREGATE_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
        "$SV_PARSER_AGGREGATE_SCRIPT"
fi

sv_generation_triage_json="$sv_parser_aggregate_state_dir/work/systemverilog_parseability_generation_counterexample_triage.json"
sv_shadow_triage_json="$sv_parser_aggregate_state_dir/work/systemverilog_closed_loop_parseability_shadow_counterexample_triage.json"
require_nonempty_file "$sv_generation_triage_json"
require_nonempty_file "$sv_shadow_triage_json"

sv_generation_failure_context_count="$(extract_json_number "$sv_generation_triage_json" '(.by_failure_context_excerpt | length)')"
sv_shadow_failure_context_count="$(extract_json_number "$sv_shadow_triage_json" '(.by_failure_context_excerpt | length)')"

# CI-PARITY-GATE-ROT.5 — the exercised/rejection counts come from the aggregate gate's own summary,
# and the report paths from its own `proof_surfaces` block, so this gate cannot judge a different
# run than the triage it just read.
sv_aggregate_summary_json="$sv_parser_aggregate_state_dir/summary.json"
require_nonempty_file "$sv_aggregate_summary_json"
sv_generation_report_json="$(extract_json_string "$sv_aggregate_summary_json" '.proof_surfaces.generation_report_json')"
sv_shadow_report_json="$(extract_json_string "$sv_aggregate_summary_json" '.proof_surfaces.shadow_report_json')"
require_nonempty_file "$sv_generation_report_json"
require_nonempty_file "$sv_shadow_report_json"

sv_generation_attempts_total="$(extract_json_number "$sv_generation_report_json" '.observed.attempts_total')"
sv_generation_rejections_total="$(extract_json_number "$sv_generation_report_json" \
    '((.observed.parser_rejections_total | numbers) + (.observed.generation_errors_total | numbers))')"
sv_shadow_attempts_total="$(extract_json_number "$sv_shadow_report_json" '.observed.attempts_total')"
sv_shadow_rejections_total="$(extract_json_number "$sv_shadow_report_json" \
    '((.observed.parser_rejections_total | numbers) + (.observed.generation_errors_total | numbers))')"

assert_failure_context_zero_is_earned \
    "generation" "$sv_generation_triage_json" \
    "$sv_generation_attempts_total" "$sv_generation_rejections_total" \
    "$sv_generation_report_json .observed"
assert_failure_context_zero_is_earned \
    "replay-shadow" "$sv_shadow_triage_json" \
    "$sv_shadow_attempts_total" "$sv_shadow_rejections_total" \
    "$sv_shadow_report_json .observed"

# ⚠️ These two feed the emitted summary. They are only meaningful when a counterexample exists —
# with a healthy zero there is nothing to preview, so record that explicitly rather than letting
# `jq -er … | strings` abort the gate on a null. The earned-zero assertion above has already proved
# the zero is legitimate; the sentinel says WHY the field is empty instead of leaving a bare "".
SV_NO_COUNTEREXAMPLE_SENTINEL="<none: zero counterexamples, earned>"
sv_generation_failure_context_example="$(
    extract_json_string "$sv_generation_triage_json" '.sample_previews[0].failure_context_excerpt' \
        2>/dev/null || printf '%s' "$SV_NO_COUNTEREXAMPLE_SENTINEL")"
sv_shadow_failure_context_example="$(
    extract_json_string "$sv_shadow_triage_json" '.sample_previews[0].failure_context_excerpt' \
        2>/dev/null || printf '%s' "$SV_NO_COUNTEREXAMPLE_SENTINEL")"

svpp_quality_state_dir="$WORK_DIR/systemverilog_preprocessor_failure_context_quality_state"
if [[ -n "$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR" ]]; then
    svpp_quality_state_dir="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
else
    run_logged_with_env_file "systemverilog_preprocessor_failure_context_quality_gate" "$SVPP_POLICY_ENV_FILE" \
        env PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$svpp_quality_state_dir" "$SVPP_QUALITY_GATE_SCRIPT"
fi

svpp_aggregate_state_dir="$WORK_DIR/sv_preprocessor_aggregate_contract_gate"
if [[ -n "$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR" ]]; then
    svpp_aggregate_state_dir="$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR"
else
    run_logged "systemverilog_preprocessor_failure_context_aggregate_contract_gate" env \
        PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$svpp_aggregate_state_dir" \
        PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR="$svpp_quality_state_dir" \
        "$SVPP_AGGREGATE_SCRIPT"
fi

svpp_triage_json="$svpp_aggregate_state_dir/work/systemverilog_preprocessor_parseability_counterexample_triage.json"
require_nonempty_file "$svpp_triage_json"

svpp_failure_context_count="$(extract_json_number "$svpp_triage_json" '(.by_failure_context_excerpt | length)')"

# CI-PARITY-GATE-ROT.5 — same earned-zero rule for the third surface.
# ⚠️ ITS SHAPE DIFFERS AND THAT IS STATED, NOT PAPERED OVER. The preprocessor parseability report
# carries `.summary.{attempts,accepted,rejected,parser_rejections}` and has NO `generation_errors`
# counter, so the rejection term for this surface is `parser_rejections` alone. The numbers are read
# from the preprocessor aggregate's own `metrics`, which is where that gate already derives them —
# not re-extracted here through a second, driftable path.
svpp_aggregate_summary_json="$svpp_aggregate_state_dir/summary.json"
require_nonempty_file "$svpp_aggregate_summary_json"
svpp_attempts_total="$(extract_json_number "$svpp_aggregate_summary_json" '.metrics.parseability_attempts_total')"
svpp_rejections_total="$(extract_json_number "$svpp_aggregate_summary_json" '.metrics.parseability_parser_rejections_total')"

assert_failure_context_zero_is_earned \
    "preprocessor" "$svpp_triage_json" \
    "$svpp_attempts_total" "$svpp_rejections_total" \
    "$svpp_aggregate_summary_json .metrics (no generation_errors counter on this surface)"

svpp_failure_context_example="$(
    extract_json_string "$svpp_triage_json" '.sample_previews[0].failure_context_excerpt' \
        2>/dev/null || printf '%s' "$SV_NO_COUNTEREXAMPLE_SENTINEL")"

{
    echo "SV Failure Context Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "svpp_policy_env_file: $SVPP_POLICY_ENV_FILE"
    echo "existing_sv_stimuli_quality_state_dir: ${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-<unset>}"
    echo "existing_sv_preprocessor_quality_state_dir: ${EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-<unset>}"
    echo "systemverilog_failure_context_quality_state_dir: $sv_quality_state_dir"
    echo "systemverilog_parser_aggregate_state_dir: $sv_parser_aggregate_state_dir"
    echo "systemverilog_generation_failure_context_excerpts: $sv_generation_failure_context_count"
    echo "systemverilog_shadow_failure_context_excerpts: $sv_shadow_failure_context_count"
    echo "systemverilog_generation_failure_context_example: $sv_generation_failure_context_example"
    echo "systemverilog_shadow_failure_context_example: $sv_shadow_failure_context_example"
    echo "systemverilog_preprocessor_failure_context_quality_state_dir: $svpp_quality_state_dir"
    echo "systemverilog_preprocessor_aggregate_state_dir: $svpp_aggregate_state_dir"
    echo "systemverilog_preprocessor_failure_context_excerpts: $svpp_failure_context_count"
    echo "systemverilog_preprocessor_failure_context_example: $svpp_failure_context_example"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
jq -n \
  --arg gate "sv_failure_context_contract_gate" \
  --argjson version 1 \
  --arg generated_at_utc "$generated_at_utc" \
  --arg state_dir "$STATE_DIR" \
  --arg summary_txt "$SUMMARY_TXT" \
  --arg summary_json "$SUMMARY_JSON" \
  --arg sv_contract_file "$SV_CONTRACT_FILE" \
  --arg svpp_policy_env_file "$SVPP_POLICY_ENV_FILE" \
  --arg existing_sv_stimuli_quality_state_dir "${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}" \
  --arg existing_sv_preprocessor_quality_state_dir "${EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}" \
  --arg systemverilog_failure_context_quality_state_dir "$sv_quality_state_dir" \
  --arg systemverilog_parser_aggregate_state_dir "$sv_parser_aggregate_state_dir" \
  --arg systemverilog_generation_counterexample_triage_json "$sv_generation_triage_json" \
  --arg systemverilog_shadow_counterexample_triage_json "$sv_shadow_triage_json" \
  --arg systemverilog_preprocessor_failure_context_quality_state_dir "$svpp_quality_state_dir" \
  --arg systemverilog_preprocessor_aggregate_state_dir "$svpp_aggregate_state_dir" \
  --arg systemverilog_preprocessor_counterexample_triage_json "$svpp_triage_json" \
  --argjson systemverilog_generation_failure_context_excerpts "$sv_generation_failure_context_count" \
  --argjson systemverilog_shadow_failure_context_excerpts "$sv_shadow_failure_context_count" \
  --argjson systemverilog_preprocessor_failure_context_excerpts "$svpp_failure_context_count" \
  --arg systemverilog_generation_failure_context_example "$sv_generation_failure_context_example" \
  --arg systemverilog_shadow_failure_context_example "$sv_shadow_failure_context_example" \
  --arg systemverilog_preprocessor_failure_context_example "$svpp_failure_context_example" \
  '{
    gate: $gate,
    version: $version,
    generated_at_utc: $generated_at_utc,
    state_dir: $state_dir,
    summary_txt: $summary_txt,
    summary_json: $summary_json,
    sv_contract_file: $sv_contract_file,
    svpp_policy_env_file: $svpp_policy_env_file,
    existing_sv_stimuli_quality_state_dir: (if $existing_sv_stimuli_quality_state_dir == "" then null else $existing_sv_stimuli_quality_state_dir end),
    existing_sv_preprocessor_quality_state_dir: (if $existing_sv_preprocessor_quality_state_dir == "" then null else $existing_sv_preprocessor_quality_state_dir end),
    proof_surfaces: {
      systemverilog_failure_context_quality_state_dir: $systemverilog_failure_context_quality_state_dir,
      systemverilog_parser_aggregate_state_dir: $systemverilog_parser_aggregate_state_dir,
      systemverilog_generation_counterexample_triage_json: $systemverilog_generation_counterexample_triage_json,
      systemverilog_shadow_counterexample_triage_json: $systemverilog_shadow_counterexample_triage_json,
      systemverilog_preprocessor_failure_context_quality_state_dir: $systemverilog_preprocessor_failure_context_quality_state_dir,
      systemverilog_preprocessor_aggregate_state_dir: $systemverilog_preprocessor_aggregate_state_dir,
      systemverilog_preprocessor_counterexample_triage_json: $systemverilog_preprocessor_counterexample_triage_json
    },
    metrics: {
      systemverilog_generation_failure_context_excerpts: $systemverilog_generation_failure_context_excerpts,
      systemverilog_shadow_failure_context_excerpts: $systemverilog_shadow_failure_context_excerpts,
      systemverilog_preprocessor_failure_context_excerpts: $systemverilog_preprocessor_failure_context_excerpts
    },
    examples: {
      systemverilog_generation_failure_context_example: $systemverilog_generation_failure_context_example,
      systemverilog_shadow_failure_context_example: $systemverilog_shadow_failure_context_example,
      systemverilog_preprocessor_failure_context_example: $systemverilog_preprocessor_failure_context_example
    }
  }' >"$SUMMARY_JSON"
require_nonempty_file "$SUMMARY_JSON"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
