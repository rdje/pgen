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
STIMULI_CONTRACT_FILE="${PGEN_REGEX_FAMILY_CONTRACT_STIMULI_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/regex_family_stimuli_contract.json}"
STIMULI_TARGET_MAX_ATTEMPTS="${PGEN_REGEX_FAMILY_CONTRACT_STIMULI_TARGET_MAX_ATTEMPTS:-10000}"

# LANG-CAPABILITY-AUDIT.10.9 — the rule-count regression RATCHET that replaced a dead floor.
#
# ⛔ THIS IS NOT A CROSS-FRONTEND FLOOR AND MUST NOT BE READ AS ONE. Until `.10.6` this gate
# asserted `rust_rule_count >= perl_rule_count` — one frontend's rule census bounded by an
# INDEPENDENT frontend's. The Perl arm is retired, so that second census no longer exists and
# the comparison is not weakened here, it is GONE. What remains is a one-sided ratchet against
# a pinned baseline: it still catches the regression the old floor existed to catch (the Rust
# frontend silently starting to see fewer rules in `grammars/regex.ebnf`), and it cannot
# degrade the way its predecessor did, because a constant right-hand side cannot go null.
#
# Restoring a real second arm is LANG-CAPABILITY-AUDIT.10.6 part 2 (the frontend<->meta-parser
# raw-AST envelope differential). Raise this pin deliberately when the grammar grows; lower it
# only with a recorded reason.
RULE_COUNT_FLOOR="${PGEN_REGEX_FAMILY_CONTRACT_RULE_COUNT_FLOOR:-276}"

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

# LANG-CAPABILITY-AUDIT.10.9 — READ A DUAL-RUN KEY OR DIE; NEVER READ ONE AS THE STRING "null".
#
# This gate used to consume four Perl-arm keys that `.10.6` removed from the producer. Plain
# `jq -r '… | .absent_key'` yields the bare word `null` for every one of them, and the three
# consumption shapes degraded differently: two `assert_equal`s failed loudly (good), one value
# was published into the summary as `null` (misleading), and the numeric floor
# `(( rust_rule_count < perl_rule_count ))` read `null` as an unset name — i.e. **0** — so a
# regression floor passed VACUOUSLY and could never fire again (measured: `a=276; b=""` ⇒ `GE`).
#
# The root cause is that the extraction cannot distinguish "the producer measured null" from
# "the producer no longer emits this key at all". This helper removes that ambiguity at the
# extraction point: an absent key, a null value, or anything other than exactly one matching
# entry aborts the gate naming the key — so the next producer schema change is loud everywhere
# instead of loud in some readers and silent in others.
dual_run_entry_value() {
    local path="$1"
    local grammar="$2"
    local key="$3"
    jq -er --arg grammar "$grammar" --arg key "$key" '
        [.entries[]? | select(.grammar == $grammar)] as $entries
        | if ($entries | length) != 1 then
              error("expected exactly 1 \($grammar) entry while reading key \"\($key)\", found \($entries | length)")
          elif ($entries[0] | has($key) | not) then
              error("dual-run \($grammar) entry has no key \"\($key)\": the producer schema changed (LANG-CAPABILITY-AUDIT.10.9)")
          elif $entries[0][$key] == null then
              error("dual-run \($grammar) key \"\($key)\" is null: the producer no longer measures it (LANG-CAPABILITY-AUDIT.10.9)")
          else
              $entries[0][$key] | tostring
          end
    ' "$path"
}

require_int() {
    local label="$1"
    local value="$2"
    if ! [[ "$value" =~ ^-?[0-9]+$ ]]; then
        echo "error: ${label} must be an integer but found '${value}'" >&2
        exit 1
    fi
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
require_file "$STIMULI_CONTRACT_FILE"
if ! [[ "$STIMULI_TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$STIMULI_TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_REGEX_FAMILY_CONTRACT_STIMULI_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$RULE_COUNT_FLOOR" =~ ^[0-9]+$ ]] || [[ "$RULE_COUNT_FLOOR" -lt 1 ]]; then
    echo "error: PGEN_REGEX_FAMILY_CONTRACT_RULE_COUNT_FLOOR must be an integer >= 1" >&2
    exit 2
fi

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
        PGEN_EBNF_STIMULI_QUALITY_CONTRACT="$STIMULI_CONTRACT_FILE" \
        PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="$STIMULI_TARGET_MAX_ATTEMPTS" \
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
frontend_regex_frontend_to_json="$(extract_csv_value "$frontend_summary_csv" "regex" "frontend_to_json")"
frontend_regex_json_to_parser="$(extract_csv_value "$frontend_summary_csv" "regex" "json_to_parser")"
frontend_regex_json_to_stimuli="$(extract_csv_value "$frontend_summary_csv" "regex" "json_to_stimuli")"
frontend_regex_overall="$(extract_csv_value "$frontend_summary_csv" "regex" "overall")"
frontend_regex_notes="$(extract_csv_value "$frontend_summary_csv" "regex" "notes")"

dual_run_strict_mode="$(extract_summary_value "$dual_run_summary_txt" "strict_mode")"
# LANG-CAPABILITY-AUDIT.10.9 — `perl_ebnf_to_json`, `perl_rule_count`,
# `raw_ast_missing_on_perl_count` and `raw_ast_missing_on_rust_count` were DELETED here rather
# than defaulted: `.10.6` retired the Perl arm, so the producer stopped emitting all four and
# no substitute measurement exists for any of them. Every remaining read goes through
# `dual_run_entry_value`, which aborts on an absent or null key.
dual_run_regex_rust_parse="$(dual_run_entry_value "$dual_run_summary_json" "regex" "rust_parse")"
dual_run_regex_rust_parse_full="$(dual_run_entry_value "$dual_run_summary_json" "regex" "rust_parse_full")"
dual_run_regex_rust_rule_count="$(dual_run_entry_value "$dual_run_summary_json" "regex" "rust_rule_count")"
dual_run_regex_raw_ast_status="$(dual_run_entry_value "$dual_run_summary_json" "regex" "raw_ast_status")"
dual_run_regex_consumed_pct="$(dual_run_entry_value "$dual_run_summary_json" "regex" "consumed_pct")"
dual_run_regex_overall="$(dual_run_entry_value "$dual_run_summary_json" "regex" "overall")"
dual_run_regex_notes="$(dual_run_entry_value "$dual_run_summary_json" "regex" "notes")"

stimuli_contract_file="$(extract_summary_value "$stimuli_summary_txt" "contract_file")"
stimuli_regex_grammar_name="$(extract_csv_value "$stimuli_summary_csv" "regex" "grammar_name")"
stimuli_regex_parseability_required="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_required")"
stimuli_regex_parseability_attempts_total="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_attempts_total")"
stimuli_regex_parseability_accepted_total="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_accepted_total")"
stimuli_regex_parseability_rejected_total="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_rejected_total")"
stimuli_regex_parseability_parser_rejections_total="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_parser_rejections_total")"
stimuli_regex_parseability_acceptance_rate_percent="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_acceptance_rate_percent")"
stimuli_regex_parseability_report_json="$(extract_csv_value "$stimuli_summary_csv" "regex" "parseability_report_json")"
stimuli_regex_parseability_counterexample_triage_json="$WORK_DIR/regex_parseability_counterexample_triage.json"
stimuli_regex_parseability_counterexample_triage_txt="$WORK_DIR/regex_parseability_counterexample_triage.txt"
stimuli_regex_initial_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "initial_targets")"
stimuli_regex_resolved_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "resolved_targets")"
stimuli_regex_final_targets="$(extract_csv_value "$stimuli_summary_csv" "regex" "final_targets")"
stimuli_regex_target_attempts="$(extract_csv_value "$stimuli_summary_csv" "regex" "target_attempts")"
stimuli_regex_stage0_successes="$(extract_csv_value "$stimuli_summary_csv" "regex" "stage0_successes")"
stimuli_regex_stage3_successes="$(extract_csv_value "$stimuli_summary_csv" "regex" "stage3_successes")"
stimuli_regex_status="$(extract_csv_value "$stimuli_summary_csv" "regex" "status")"

assert_equal "frontend regex frontend_to_json" "pass" "$frontend_regex_frontend_to_json"
assert_equal "frontend regex json_to_parser" "pass" "$frontend_regex_json_to_parser"
assert_equal "frontend regex json_to_stimuli" "pass" "$frontend_regex_json_to_stimuli"
assert_equal "frontend regex overall" "pass" "$frontend_regex_overall"

assert_equal "dual-run regex rust_parse" "pass" "$dual_run_regex_rust_parse"
assert_equal "dual-run regex rust_parse_full" "pass" "$dual_run_regex_rust_parse_full"
assert_equal "dual-run regex overall" "pass" "$dual_run_regex_overall"

# LANG-CAPABILITY-AUDIT.10.9 — an EXACT match, not the old allowlist with `exported` appended.
# The producer writes exactly two values for this key: `exported` when the hand-written
# frontend's raw-AST envelope was produced, and `skip` when that export FAILED. So `exported`
# is its only success value and an allowlist would only re-admit failure. The retired
# `parity|perl_under_reports` pair described Perl-vs-Rust agreement, which no longer exists;
# in particular `perl_under_reports` was a tolerated PASS that accepted the Perl arm being
# blind to 25 of `regex.ebnf`'s 276 rules.
assert_equal "dual-run regex raw_ast_status" "exported" "$dual_run_regex_raw_ast_status"

# ⛔ THE CROSS-FRONTEND RULE-COUNT FLOOR IS RETIRED, NOT WEAKENED — see RULE_COUNT_FLOOR above.
# What follows is a one-sided regression ratchet against a pinned baseline, and the gate says
# so in its own summary (`dual_run_regex_cross_frontend_floor: retired`) so that no downstream
# reader can infer a second frontend from the presence of a numeric floor.
require_int "dual-run regex rust_rule_count" "$dual_run_regex_rust_rule_count"
assert_int_ge "dual-run regex rust_rule_count" "$dual_run_regex_rust_rule_count" "$RULE_COUNT_FLOOR"

assert_equal "stimuli regex grammar_name" "regex" "$stimuli_regex_grammar_name"
assert_equal "stimuli regex parseability_required" "1" "$stimuli_regex_parseability_required"
assert_equal "stimuli regex status" "pass" "$stimuli_regex_status"
assert_int_ge "stimuli regex parseability_attempts_total" "$stimuli_regex_parseability_attempts_total" 1
assert_int_ge "stimuli regex parseability_accepted_total" "$stimuli_regex_parseability_accepted_total" 1
assert_int_ge "stimuli regex parseability_rejected_total" "$stimuli_regex_parseability_rejected_total" 0
assert_int_ge "stimuli regex parseability_parser_rejections_total" "$stimuli_regex_parseability_parser_rejections_total" 0
require_nonempty_file "$stimuli_regex_parseability_report_json"

jq \
    --arg source_report "$stimuli_regex_parseability_report_json" \
    '(.stages | [.[]?.counterexamples[]?]) as $counterexamples
    | {
        source_report: $source_report,
        counterexamples_captured_total: ($counterexamples | length),
        by_stage: (
            $counterexamples
            | map(.stage)
            | group_by(.)
            | map({
                stage: .[0],
                count: length
            })
        ),
        by_shrunk_sample: (
            $counterexamples
            | map(.shrunk_sample)
            | group_by(.)
            | map({
                shrunk_sample: .[0],
                count: length
            })
        ),
        by_parser_error: (
            $counterexamples
            | map(.parser_error // "<none>")
            | group_by(.)
            | map({
                parser_error: .[0],
                count: length
            })
        ),
        by_failure_location: (
            $counterexamples
            | map([(.failure_line // -1), (.failure_column // -1)])
            | group_by(.)
            | map({
                failure_line: .[0][0],
                failure_column: .[0][1],
                count: length
            })
        ),
        by_failure_line_excerpt: (
            $counterexamples
            | map(.failure_line_excerpt // "<none>")
            | group_by(.)
            | map({
                failure_line_excerpt: .[0],
                count: length
            })
        ),
        by_failure_context_excerpt: (
            $counterexamples
            | map(.failure_context_excerpt // "<none>")
            | group_by(.)
            | map({
                failure_context_excerpt: .[0],
                count: length
            })
        ),
        sample_previews: (
            $counterexamples[:5]
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
    }' \
    "$stimuli_regex_parseability_report_json" >"$stimuli_regex_parseability_counterexample_triage_json"
require_nonempty_file "$stimuli_regex_parseability_counterexample_triage_json"

{
    echo "Regex Parseability Counterexample Triage"
    echo "source_report: $stimuli_regex_parseability_report_json"
    jq -r '.by_stage[]? | "stage_count[\(.stage)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
    jq -r '.by_shrunk_sample[]? | "shrunk_sample_count[\(.shrunk_sample | @json)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
    jq -r '.by_parser_error[]? | "parser_error_count[\(.parser_error | @json)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
    jq -r '.by_failure_location[]? | "failure_location[\(.failure_line):\(.failure_column)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
    jq -r '.by_failure_line_excerpt[]? | "failure_line_excerpt_count[\(.failure_line_excerpt | @json)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
    jq -r '.by_failure_context_excerpt[]? | "failure_context_excerpt_count[\(.failure_context_excerpt | @json)]: \(.count)"' "$stimuli_regex_parseability_counterexample_triage_json"
} >"$stimuli_regex_parseability_counterexample_triage_txt"
require_nonempty_file "$stimuli_regex_parseability_counterexample_triage_txt"

stimuli_regex_parseability_counterexamples_captured_total="$(jq -er '.counterexamples_captured_total | numbers' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_unique_shrunk_samples="$(jq -er '(.by_shrunk_sample | length) | numbers' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_unique_failure_locations="$(jq -er '(.by_failure_location | length) | numbers' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_unique_failure_line_excerpts="$(jq -er '(.by_failure_line_excerpt | length) | numbers' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_unique_failure_context_excerpts="$(jq -er '(.by_failure_context_excerpt | length) | numbers' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_stage="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].stage) else "<none>" end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_stage_count="$(jq -er 'if (.by_stage | length) > 0 then (.by_stage | sort_by(-.count, .stage) | .[0].count) else 0 end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_shrunk_sample="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].shrunk_sample) else "<none>" end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_shrunk_sample_count="$(jq -er 'if (.by_shrunk_sample | length) > 0 then (.by_shrunk_sample | sort_by(-.count, .shrunk_sample) | .[0].count) else 0 end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_parser_error="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].parser_error) else "<none>" end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_parser_error_count="$(jq -er 'if (.by_parser_error | length) > 0 then (.by_parser_error | sort_by(-.count, .parser_error) | .[0].count) else 0 end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_failure_location="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0] | "\(.failure_line):\(.failure_column)") else "<none>" end' "$stimuli_regex_parseability_counterexample_triage_json")"
stimuli_regex_parseability_counterexample_primary_failure_location_count="$(jq -er 'if (.by_failure_location | length) > 0 then (.by_failure_location | sort_by(-.count, .failure_line, .failure_column) | .[0].count) else 0 end' "$stimuli_regex_parseability_counterexample_triage_json")"

assert_int_ge "stimuli regex initial_targets" "$stimuli_regex_initial_targets" 1
assert_int_ge "stimuli regex resolved_targets" "$stimuli_regex_resolved_targets" 1
assert_int_ge "stimuli regex target_attempts" "$stimuli_regex_target_attempts" 1
assert_int_ge "stimuli regex stage0_successes" "$stimuli_regex_stage0_successes" 1
if (( stimuli_regex_parseability_parser_rejections_total > stimuli_regex_parseability_rejected_total )); then
    echo "error: stimuli regex parseability parser_rejections_total ${stimuli_regex_parseability_parser_rejections_total} exceeds rejected_total ${stimuli_regex_parseability_rejected_total}" >&2
    exit 1
fi
if (( stimuli_regex_parseability_parser_rejections_total > 0 && stimuli_regex_parseability_counterexamples_captured_total < 1 )); then
    echo "error: stimuli regex parseability triage captured no counterexamples despite parser_rejections_total ${stimuli_regex_parseability_parser_rejections_total}" >&2
    exit 1
fi
# CI-PARITY-GATE-ROT.9: this was a strict `resolved + final == initial`. The equality is a
# true statement about the pipeline, but it is NOT sound as a gate: stage 3 generates a
# further sample, and a target IT resolves makes the sum fall short on a perfectly healthy
# run. It is now split into the two checks it was conflating.
#
# (1) SOUNDNESS. The drive-plus-witness resolved set and the still-actionable set are
#     disjoint subsets of the initial set, so their sizes cannot exceed it. A sum that does
#     means double-counting or resolution reported for a target that was never in the set.
if (( stimuli_regex_resolved_targets + stimuli_regex_final_targets > stimuli_regex_initial_targets )); then
    echo "error: stimuli regex target accounting unsound (${stimuli_regex_resolved_targets} + ${stimuli_regex_final_targets} > ${stimuli_regex_initial_targets})" >&2
    exit 1
fi
# (2) PROGRESS — the check the old equality could not make. `resolved=0, final=initial` is a
#     closed loop that achieved literally nothing, and the strict equality PASSED it. With
#     targets to close, the loop must close at least one.
if (( stimuli_regex_final_targets >= stimuli_regex_initial_targets )); then
    echo "error: stimuli regex closed loop resolved no targets (final ${stimuli_regex_final_targets} >= initial ${stimuli_regex_initial_targets})" >&2
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
    echo "summary_json: $SUMMARY_JSON"
    echo "frontend_state_dir: $frontend_state_dir"
    echo "frontend_summary_txt: $frontend_summary_txt"
    echo "frontend_summary_csv: $frontend_summary_csv"
    echo "frontend_impl: $frontend_impl"
    echo "frontend_regex_frontend_to_json: $frontend_regex_frontend_to_json"
    echo "frontend_regex_json_to_parser: $frontend_regex_json_to_parser"
    echo "frontend_regex_json_to_stimuli: $frontend_regex_json_to_stimuli"
    echo "frontend_regex_overall: $frontend_regex_overall"
    echo "frontend_regex_notes: $frontend_regex_notes"
    echo "dual_run_state_dir: $dual_run_state_dir"
    echo "dual_run_summary_txt: $dual_run_summary_txt"
    echo "dual_run_summary_csv: $dual_run_summary_csv"
    echo "dual_run_summary_json: $dual_run_summary_json"
    echo "dual_run_regex_rust_parse: $dual_run_regex_rust_parse"
    echo "dual_run_regex_rust_parse_full: $dual_run_regex_rust_parse_full"
    echo "dual_run_regex_rust_rule_count: $dual_run_regex_rust_rule_count"
    echo "dual_run_regex_rust_rule_count_floor: $RULE_COUNT_FLOOR"
    echo "dual_run_regex_cross_frontend_floor: retired"
    echo "dual_run_regex_raw_ast_status: $dual_run_regex_raw_ast_status"
    echo "dual_run_regex_consumed_pct: $dual_run_regex_consumed_pct"
    echo "dual_run_regex_overall: $dual_run_regex_overall"
    echo "dual_run_regex_notes: $dual_run_regex_notes"
    echo "stimuli_state_dir: $stimuli_state_dir"
    echo "stimuli_summary_txt: $stimuli_summary_txt"
    echo "stimuli_summary_csv: $stimuli_summary_csv"
    echo "stimuli_contract_file: $stimuli_contract_file"
    echo "stimuli_target_max_attempts: $STIMULI_TARGET_MAX_ATTEMPTS"
    echo "stimuli_parseability_report_json: $stimuli_regex_parseability_report_json"
    echo "stimuli_parseability_counterexample_triage_json: $stimuli_regex_parseability_counterexample_triage_json"
    echo "stimuli_parseability_counterexample_triage_txt: $stimuli_regex_parseability_counterexample_triage_txt"
    echo "stimuli_regex_parseability_required: $stimuli_regex_parseability_required"
    echo "stimuli_regex_parseability_attempts_total: $stimuli_regex_parseability_attempts_total"
    echo "stimuli_regex_parseability_accepted_total: $stimuli_regex_parseability_accepted_total"
    echo "stimuli_regex_parseability_rejected_total: $stimuli_regex_parseability_rejected_total"
    echo "stimuli_regex_parseability_parser_rejections_total: $stimuli_regex_parseability_parser_rejections_total"
    echo "stimuli_regex_parseability_acceptance_rate_percent: $stimuli_regex_parseability_acceptance_rate_percent"
    echo "stimuli_regex_parseability_counterexamples_captured_total: $stimuli_regex_parseability_counterexamples_captured_total"
    echo "stimuli_regex_parseability_counterexample_unique_shrunk_samples: $stimuli_regex_parseability_counterexample_unique_shrunk_samples"
    echo "stimuli_regex_parseability_counterexample_unique_failure_locations: $stimuli_regex_parseability_counterexample_unique_failure_locations"
    echo "stimuli_regex_parseability_counterexample_unique_failure_line_excerpts: $stimuli_regex_parseability_counterexample_unique_failure_line_excerpts"
    echo "stimuli_regex_parseability_counterexample_unique_failure_context_excerpts: $stimuli_regex_parseability_counterexample_unique_failure_context_excerpts"
    echo "stimuli_regex_parseability_counterexample_primary_stage: $stimuli_regex_parseability_counterexample_primary_stage"
    echo "stimuli_regex_parseability_counterexample_primary_stage_count: $stimuli_regex_parseability_counterexample_primary_stage_count"
    echo "stimuli_regex_parseability_counterexample_primary_shrunk_sample: $stimuli_regex_parseability_counterexample_primary_shrunk_sample"
    echo "stimuli_regex_parseability_counterexample_primary_shrunk_sample_count: $stimuli_regex_parseability_counterexample_primary_shrunk_sample_count"
    echo "stimuli_regex_parseability_counterexample_primary_parser_error: $stimuli_regex_parseability_counterexample_primary_parser_error"
    echo "stimuli_regex_parseability_counterexample_primary_parser_error_count: $stimuli_regex_parseability_counterexample_primary_parser_error_count"
    echo "stimuli_regex_parseability_counterexample_primary_failure_location: $stimuli_regex_parseability_counterexample_primary_failure_location"
    echo "stimuli_regex_parseability_counterexample_primary_failure_location_count: $stimuli_regex_parseability_counterexample_primary_failure_location_count"
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
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg frontend_state_dir "$frontend_state_dir" \
    --arg frontend_summary_txt "$frontend_summary_txt" \
    --arg frontend_summary_csv "$frontend_summary_csv" \
    --arg frontend_impl "$frontend_impl" \
    --arg frontend_regex_frontend_to_json "$frontend_regex_frontend_to_json" \
    --arg frontend_regex_json_to_parser "$frontend_regex_json_to_parser" \
    --arg frontend_regex_json_to_stimuli "$frontend_regex_json_to_stimuli" \
    --arg frontend_regex_overall "$frontend_regex_overall" \
    --arg frontend_regex_notes "$frontend_regex_notes" \
    --arg dual_run_state_dir "$dual_run_state_dir" \
    --arg dual_run_summary_txt "$dual_run_summary_txt" \
    --arg dual_run_summary_csv "$dual_run_summary_csv" \
    --arg dual_run_summary_json "$dual_run_summary_json" \
    --arg dual_run_regex_rust_parse "$dual_run_regex_rust_parse" \
    --arg dual_run_regex_rust_parse_full "$dual_run_regex_rust_parse_full" \
    --argjson dual_run_regex_rust_rule_count "$dual_run_regex_rust_rule_count" \
    --argjson dual_run_regex_rust_rule_count_floor "$RULE_COUNT_FLOOR" \
    --arg dual_run_regex_cross_frontend_floor "retired" \
    --arg dual_run_regex_raw_ast_status "$dual_run_regex_raw_ast_status" \
    --argjson dual_run_regex_consumed_pct "$dual_run_regex_consumed_pct" \
    --arg dual_run_regex_overall "$dual_run_regex_overall" \
    --arg dual_run_regex_notes "$dual_run_regex_notes" \
    --arg stimuli_state_dir "$stimuli_state_dir" \
    --arg stimuli_summary_txt "$stimuli_summary_txt" \
    --arg stimuli_summary_csv "$stimuli_summary_csv" \
    --arg stimuli_contract_file "$stimuli_contract_file" \
    --argjson stimuli_target_max_attempts "$STIMULI_TARGET_MAX_ATTEMPTS" \
    --arg stimuli_parseability_report_json "$stimuli_regex_parseability_report_json" \
    --arg stimuli_parseability_counterexample_triage_json "$stimuli_regex_parseability_counterexample_triage_json" \
    --arg stimuli_parseability_counterexample_triage_txt "$stimuli_regex_parseability_counterexample_triage_txt" \
    --argjson stimuli_regex_parseability_required "$stimuli_regex_parseability_required" \
    --argjson stimuli_regex_parseability_attempts_total "$stimuli_regex_parseability_attempts_total" \
    --argjson stimuli_regex_parseability_accepted_total "$stimuli_regex_parseability_accepted_total" \
    --argjson stimuli_regex_parseability_rejected_total "$stimuli_regex_parseability_rejected_total" \
    --argjson stimuli_regex_parseability_parser_rejections_total "$stimuli_regex_parseability_parser_rejections_total" \
    --argjson stimuli_regex_parseability_acceptance_rate_percent "$stimuli_regex_parseability_acceptance_rate_percent" \
    --argjson stimuli_regex_parseability_counterexamples_captured_total "$stimuli_regex_parseability_counterexamples_captured_total" \
    --argjson stimuli_regex_parseability_counterexample_unique_shrunk_samples "$stimuli_regex_parseability_counterexample_unique_shrunk_samples" \
    --argjson stimuli_regex_parseability_counterexample_unique_failure_locations "$stimuli_regex_parseability_counterexample_unique_failure_locations" \
    --argjson stimuli_regex_parseability_counterexample_unique_failure_line_excerpts "$stimuli_regex_parseability_counterexample_unique_failure_line_excerpts" \
    --argjson stimuli_regex_parseability_counterexample_unique_failure_context_excerpts "$stimuli_regex_parseability_counterexample_unique_failure_context_excerpts" \
    --arg stimuli_regex_parseability_counterexample_primary_stage "$stimuli_regex_parseability_counterexample_primary_stage" \
    --argjson stimuli_regex_parseability_counterexample_primary_stage_count "$stimuli_regex_parseability_counterexample_primary_stage_count" \
    --arg stimuli_regex_parseability_counterexample_primary_shrunk_sample "$stimuli_regex_parseability_counterexample_primary_shrunk_sample" \
    --argjson stimuli_regex_parseability_counterexample_primary_shrunk_sample_count "$stimuli_regex_parseability_counterexample_primary_shrunk_sample_count" \
    --arg stimuli_regex_parseability_counterexample_primary_parser_error "$stimuli_regex_parseability_counterexample_primary_parser_error" \
    --argjson stimuli_regex_parseability_counterexample_primary_parser_error_count "$stimuli_regex_parseability_counterexample_primary_parser_error_count" \
    --arg stimuli_regex_parseability_counterexample_primary_failure_location "$stimuli_regex_parseability_counterexample_primary_failure_location" \
    --argjson stimuli_regex_parseability_counterexample_primary_failure_location_count "$stimuli_regex_parseability_counterexample_primary_failure_location_count" \
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
      summary_txt: $summary_txt,
      summary_json: $summary_json,
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
        stimuli_summary_csv: $stimuli_summary_csv,
        stimuli_parseability_report_json: $stimuli_parseability_report_json,
        stimuli_parseability_counterexample_triage_json: $stimuli_parseability_counterexample_triage_json,
        stimuli_parseability_counterexample_triage_txt: $stimuli_parseability_counterexample_triage_txt
      },
      metrics: {
        frontend_impl: $frontend_impl,
        frontend_regex_frontend_to_json: $frontend_regex_frontend_to_json,
        frontend_regex_json_to_parser: $frontend_regex_json_to_parser,
        frontend_regex_json_to_stimuli: $frontend_regex_json_to_stimuli,
        frontend_regex_overall: $frontend_regex_overall,
        frontend_regex_notes: $frontend_regex_notes,
        dual_run_regex_rust_parse: $dual_run_regex_rust_parse,
        dual_run_regex_rust_parse_full: $dual_run_regex_rust_parse_full,
        dual_run_regex_rust_rule_count: $dual_run_regex_rust_rule_count,
        dual_run_regex_rust_rule_count_floor: $dual_run_regex_rust_rule_count_floor,
        dual_run_regex_cross_frontend_floor: $dual_run_regex_cross_frontend_floor,
        dual_run_regex_raw_ast_status: $dual_run_regex_raw_ast_status,
        dual_run_regex_consumed_pct: $dual_run_regex_consumed_pct,
        dual_run_regex_overall: $dual_run_regex_overall,
        dual_run_regex_notes: $dual_run_regex_notes,
        stimuli_contract_file: $stimuli_contract_file,
        stimuli_target_max_attempts: $stimuli_target_max_attempts,
        stimuli_regex_parseability_required: $stimuli_regex_parseability_required,
        stimuli_regex_parseability_attempts_total: $stimuli_regex_parseability_attempts_total,
        stimuli_regex_parseability_accepted_total: $stimuli_regex_parseability_accepted_total,
        stimuli_regex_parseability_rejected_total: $stimuli_regex_parseability_rejected_total,
        stimuli_regex_parseability_parser_rejections_total: $stimuli_regex_parseability_parser_rejections_total,
        stimuli_regex_parseability_acceptance_rate_percent: $stimuli_regex_parseability_acceptance_rate_percent,
        stimuli_regex_parseability_counterexamples_captured_total: $stimuli_regex_parseability_counterexamples_captured_total,
        stimuli_regex_parseability_counterexample_unique_shrunk_samples: $stimuli_regex_parseability_counterexample_unique_shrunk_samples,
        stimuli_regex_parseability_counterexample_unique_failure_locations: $stimuli_regex_parseability_counterexample_unique_failure_locations,
        stimuli_regex_parseability_counterexample_unique_failure_line_excerpts: $stimuli_regex_parseability_counterexample_unique_failure_line_excerpts,
        stimuli_regex_parseability_counterexample_unique_failure_context_excerpts: $stimuli_regex_parseability_counterexample_unique_failure_context_excerpts,
        stimuli_regex_parseability_counterexample_primary_stage: $stimuli_regex_parseability_counterexample_primary_stage,
        stimuli_regex_parseability_counterexample_primary_stage_count: $stimuli_regex_parseability_counterexample_primary_stage_count,
        stimuli_regex_parseability_counterexample_primary_shrunk_sample: $stimuli_regex_parseability_counterexample_primary_shrunk_sample,
        stimuli_regex_parseability_counterexample_primary_shrunk_sample_count: $stimuli_regex_parseability_counterexample_primary_shrunk_sample_count,
        stimuli_regex_parseability_counterexample_primary_parser_error: $stimuli_regex_parseability_counterexample_primary_parser_error,
        stimuli_regex_parseability_counterexample_primary_parser_error_count: $stimuli_regex_parseability_counterexample_primary_parser_error_count,
        stimuli_regex_parseability_counterexample_primary_failure_location: $stimuli_regex_parseability_counterexample_primary_failure_location,
        stimuli_regex_parseability_counterexample_primary_failure_location_count: $stimuli_regex_parseability_counterexample_primary_failure_location_count,
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
