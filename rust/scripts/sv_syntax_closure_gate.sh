#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_SYNTAX_CLOSURE_STATE_DIR:-$RUST_DIR/target/sv_syntax_closure_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_JSON="$STATE_DIR/summary.json"
SUMMARY_TXT="$STATE_DIR/summary.txt"

CONTRACT_FILE="${PGEN_SV_SYNTAX_CLOSURE_CONTRACT:-$RUST_DIR/test_data/grammar_quality/systemverilog_syntax_closure_contract.json}"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

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

mkdir -p "$LOG_DIR" "$WORK_DIR"

require_tool jq
require_file "$CONTRACT_FILE"

# --- BASELINE IDENTITY: is this contract still describing THIS tree? -------------------
#
# Doctrine `BASELINE-IDENTITY` (SV-CORPUS-GRAD.13c.2x.2, redesigned by `.13c.2x.4`).
#
# ⛔⛔ THE IDENTITY DISAMBIGUATES THIS GATE'S VERDICT; IT DOES NOT GATE THE MEASUREMENT.
# The first cut refused whenever an input had moved, and that was a design error: it refused
# precisely in the case where RUNNING would have resolved everything for free, and — because the
# same enforcer runs from .githooks/pre-commit — it blocked every commit in the repository. The
# full matrix, and only one cell needs a person:
#
#   identity   constraints   verdict
#   fresh      green         PASS
#   fresh      RED           REAL REGRESSION — the sharp verdict this doctrine exists to produce
#   STALE      green         the baseline was stale and still CORRECT -> this run RE-STAMPS it
#   STALE      RED           AMBIGUOUS -> refuse, naming both halves
#   UNCONFIRMED  —           refuse BEFORE measuring: a person already ruled the numbers wrong,
#                            so ~90 s of work would teach nothing
#
# ⭐ Because a green run re-stamps itself, this gate IS the confirming run, and the env-gated
# escape the first design needed (`PGEN_SV_SYNTAX_CLOSURE_CONFIRMING_RUN`) is DELETED — the
# redesign removes a bypass surface rather than adding one.
identity_rc=0
identity_stale=0
"$ROOT_DIR/scripts/check_baseline_identity.sh" --verify "${CONTRACT_FILE#"$ROOT_DIR/"}" \
    || identity_rc=$?
case "$identity_rc" in
    0) ;;
    1) identity_stale=1 ;;
    3)
        {
            echo ""
            echo "sv_syntax_closure_gate: REFUSING TO MEASURE — the contract's expectations are"
            echo "  UNCONFIRMED (see above). A person has already recorded that these numbers do"
            echo "  not describe the recorded tree, so measuring cannot resolve anything."
        } >&2
        exit 2
        ;;
    *)
        {
            echo ""
            echo "sv_syntax_closure_gate: REFUSING TO MEASURE — the contract's identity could not"
            echo "  be read (rc=$identity_rc). Resolve the refusal above first."
        } >&2
        exit 2
        ;;
esac

contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$CONTRACT_FILE")"
entry_rule="$(jq -er '.entry_rule | strings' "$CONTRACT_FILE")"
ebnf_path_rel="$(jq -er '.ebnf_path | strings' "$CONTRACT_FILE")"
stimuli_seed="$(jq -er '(.pipeline.stimuli_seed // 24001) | numbers' "$CONTRACT_FILE")"
stimuli_count="$(jq -er '(.pipeline.stimuli_count // 1) | numbers' "$CONTRACT_FILE")"
gap_report_threshold="$(jq -er '(.pipeline.gap_report_threshold // 1) | numbers' "$CONTRACT_FILE")"
min_total_rules="$(jq -er '(.constraints.min_total_rules // 1) | numbers' "$CONTRACT_FILE")"
max_unresolved_rule_references="$(jq -er '(.constraints.max_unresolved_rule_references // 0) | numbers' "$CONTRACT_FILE")"
require_unique_rule_names="$(jq -er 'if (.constraints.require_unique_rule_names // true) then 1 else 0 end' "$CONTRACT_FILE")"
require_entry_rule_defined="$(jq -er 'if (.constraints.require_entry_rule_defined // true) then 1 else 0 end' "$CONTRACT_FILE")"
max_unreachable_rules="$(jq -er '(.constraints.max_unreachable_rules // 999999) | numbers' "$CONTRACT_FILE")"
max_unreachable_branches="$(jq -er '(.constraints.max_unreachable_branches // 999999) | numbers' "$CONTRACT_FILE")"
min_reachable_rules="$(jq -er '(.constraints.min_reachable_rules // 1) | numbers' "$CONTRACT_FILE")"

# --- the NAMED unreachable allow-list (SV-CORPUS-GRAD.13c.2x.3) ------------------------
#
# ⛔ A BARE CEILING RECORDS NO REASON, AND THAT IS WHY THIS ONE ROTTED. `max_unreachable_rules=0`
# was authored 2026-06-17; `rust/src/ast_pipeline/indirect_lr_elimination.rs` — the pass whose
# residue breaks it — did not exist until 2026-08-13, two months later, and 69 grammar revisions
# passed with nobody able to tell a stale threshold from a real regression. A NAMED list says which
# rules and why, so the next divergence names itself. Shape copied from the in-repo precedent,
# `systemverilog_preprocessor_zero_plausible_gap_proof_contract.json`, rather than invented.
#
# ⚠️ AND THE METRIC'S PREDICATE IS NOW DECLARED, because `unreachable_rules` is ONE NAME FOR TWO.
# This gate's number comes from the syntax-probe gap report and is ENTRY-SCOPED — measured 3 / 14 /
# 1058 for `sv_multi_entry_root` / `systemverilog_file` / `library_text` on one grammar. The lint's
# same-named counter treats an unreferenced orphan as a ROOT by design
# (`grammar_wellformedness.rs:341`) and reports 0. Neither is wrong; a contract that does not say
# which it means sends a reader to the wrong instrument.
allowed_unreachable_rules_json="$(jq -cer '(.constraints.allowed_unreachable_rules // null)
                                           | if . == null then null else sort end' "$CONTRACT_FILE")"
required_unreachable_rule_reason="$(jq -r '.constraints.required_unreachable_rule_reason // ""' "$CONTRACT_FILE")"
if [[ "$allowed_unreachable_rules_json" != "null" ]]; then
    if [[ -z "$required_unreachable_rule_reason" ]]; then
        echo "error: constraints.allowed_unreachable_rules is present but constraints.required_unreachable_rule_reason is not" >&2
        exit 2
    fi
    allowed_unreachable_count="$(jq -er 'length | numbers' <<<"$allowed_unreachable_rules_json")"
    # ⛔ THE TWO FIELDS CANNOT DISAGREE. A named list beside a ceiling that permits a different
    # number is two contracts in one file, and the reader has no way to know which binds.
    if (( allowed_unreachable_count != max_unreachable_rules )); then
        echo "error: constraints.max_unreachable_rules=${max_unreachable_rules} disagrees with" >&2
        echo "       the ${allowed_unreachable_count}-entry constraints.allowed_unreachable_rules list" >&2
        exit 2
    fi
fi

if ! [[ "$stimuli_seed" =~ ^[0-9]+$ ]]; then
    echo "error: pipeline.stimuli_seed must be integer >= 0" >&2
    exit 2
fi
if ! [[ "$stimuli_count" =~ ^[0-9]+$ ]] || [[ "$stimuli_count" -lt 1 ]]; then
    echo "error: pipeline.stimuli_count must be integer >= 1" >&2
    exit 2
fi
if ! [[ "$gap_report_threshold" =~ ^[0-9]+$ ]] || [[ "$gap_report_threshold" -lt 1 ]]; then
    echo "error: pipeline.gap_report_threshold must be integer >= 1" >&2
    exit 2
fi
if ! [[ "$min_total_rules" =~ ^[0-9]+$ ]] || [[ "$min_total_rules" -lt 1 ]]; then
    echo "error: constraints.min_total_rules must be integer >= 1" >&2
    exit 2
fi
if ! [[ "$max_unresolved_rule_references" =~ ^[0-9]+$ ]]; then
    echo "error: constraints.max_unresolved_rule_references must be integer >= 0" >&2
    exit 2
fi
if ! [[ "$max_unreachable_rules" =~ ^[0-9]+$ ]]; then
    echo "error: constraints.max_unreachable_rules must be integer >= 0" >&2
    exit 2
fi
if ! [[ "$max_unreachable_branches" =~ ^[0-9]+$ ]]; then
    echo "error: constraints.max_unreachable_branches must be integer >= 0" >&2
    exit 2
fi
if ! [[ "$min_reachable_rules" =~ ^[0-9]+$ ]] || [[ "$min_reachable_rules" -lt 1 ]]; then
    echo "error: constraints.min_reachable_rules must be integer >= 1" >&2
    exit 2
fi

grammar_file="$ROOT_DIR/$ebnf_path_rel"
grammar_json="$WORK_DIR/${grammar_name}.json"
parser_out="$WORK_DIR/${grammar_name}_parser.rs"
stimuli_out="$WORK_DIR/${grammar_name}_syntax_probe_stimuli.txt"
coverage_json="$WORK_DIR/${grammar_name}_syntax_probe_coverage.json"
gap_json="$WORK_DIR/${grammar_name}_syntax_probe_gap.json"
gap_txt="$WORK_DIR/${grammar_name}_syntax_probe_gap.txt"
unresolved_refs_file="$WORK_DIR/${grammar_name}_unresolved_rule_references.txt"

require_file "$grammar_file"

echo "==> ${grammar_name} syntax closure gate"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_name: $grammar_name"
echo "entry_rule: $entry_rule"
echo "grammar_file: $grammar_file"
echo "grammar_raw_ast_json: $grammar_json"
echo "generated_parser_file: $parser_out"
echo "stimuli_seed: $stimuli_seed"
echo "stimuli_count: $stimuli_count"
echo "gap_report_threshold: $gap_report_threshold"

run_logged_rust "build_ast_pipeline" cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary missing at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

run_logged "frontend_rust_raw_ast_export" \
    "$AST_PIPELINE_BIN" "$grammar_file" --emit-raw-ast-json "$grammar_json"
require_nonempty_file "$grammar_json"

run_logged "generate_parser" \
    "$AST_PIPELINE_BIN" "$grammar_file" \
    --generate-parser \
    --emit-raw-ast-json "$grammar_json" \
    --eliminate-left-recursion \
    --output "$parser_out"
require_nonempty_file "$parser_out"

run_logged "syntax_probe_stimuli_gap" \
    "$AST_PIPELINE_BIN" "$grammar_json" \
    --generate-stimuli \
    --count "$stimuli_count" \
    --seed "$stimuli_seed" \
    --entry-rule "$entry_rule" \
    --output "$stimuli_out" \
    --coverage-output "$coverage_json" \
    --gap-report-json "$gap_json" \
    --gap-report-text "$gap_txt" \
    --gap-report-threshold "$gap_report_threshold"
require_nonempty_file "$coverage_json"
require_nonempty_file "$gap_json"
require_nonempty_file "$gap_txt"

jq -r '
  .raw_ast as $ast |
  ($ast | map(.[0][1])) as $defs |
  [ $ast[] | .[1:][]? | select(.[0] == "rule_reference") | .[1] ] as $refs |
  ($refs | unique | map(select(($defs | index(.)) == null)) | .[])' \
  "$grammar_json" >"$unresolved_refs_file"

defined_rule_count="$(jq -er '.raw_ast | length | numbers' "$grammar_json")"
unique_rule_name_count="$(jq -er '.raw_ast | map(.[0][1]) | unique | length | numbers' "$grammar_json")"
rule_reference_count="$(jq -er '[.raw_ast[] | .[1:][]? | select(.[0] == "rule_reference")] | length | numbers' "$grammar_json")"
unique_rule_reference_count="$(jq -er '[.raw_ast[] | .[1:][]? | select(.[0] == "rule_reference") | .[1]] | unique | length | numbers' "$grammar_json")"
unresolved_rule_reference_count="$(grep -c '.' "$unresolved_refs_file" || true)"
entry_rule_defined=0
if jq -er --arg entry "$entry_rule" '.raw_ast | map(.[0][1]) | index($entry)' "$grammar_json" >/dev/null; then
    entry_rule_defined=1
fi

reachable_rules="$(jq -er '.summary.reachable_rules | numbers' "$gap_json")"
unreachable_rules="$(jq -er '.summary.unreachable_rules | numbers' "$gap_json")"
reachable_branches="$(jq -er '.summary.reachable_branches | numbers' "$gap_json")"
unreachable_branches="$(jq -er '.summary.unreachable_branches | numbers' "$gap_json")"
target_debt_count="$(jq -er '(.targets // []) | length | numbers' "$gap_json")"

failures=0
failure_notes=()
if (( defined_rule_count < min_total_rules )); then
    failures=$((failures + 1))
    failure_notes+=("defined_rule_count=${defined_rule_count} < min_total_rules=${min_total_rules}")
fi
if (( unresolved_rule_reference_count > max_unresolved_rule_references )); then
    failures=$((failures + 1))
    failure_notes+=("unresolved_rule_reference_count=${unresolved_rule_reference_count} > max_unresolved_rule_references=${max_unresolved_rule_references}")
fi
if [[ "$require_unique_rule_names" -eq 1 ]] && (( unique_rule_name_count != defined_rule_count )); then
    failures=$((failures + 1))
    failure_notes+=("duplicate rule names detected: unique=${unique_rule_name_count}, total=${defined_rule_count}")
fi
if [[ "$require_entry_rule_defined" -eq 1 ]] && [[ "$entry_rule_defined" -ne 1 ]]; then
    failures=$((failures + 1))
    failure_notes+=("entry_rule '${entry_rule}' is not defined")
fi
if [[ "$allowed_unreachable_rules_json" != "null" ]]; then
    # ⭐ SET EQUALITY, BOTH WAYS — the register's own two-sided rule, one surface over. An observed
    # rule missing from the list is an unexplained unreachability; a listed rule that is no longer
    # observed is a DEAD EXEMPTION, and the SVPP precedent refuses those for the same reason.
    observed_unreachable_rules_json="$(jq -cer '[.unreachable_rule_debt[]?.rule_name] | sort' "$gap_json")"
    if [[ "$observed_unreachable_rules_json" != "$allowed_unreachable_rules_json" ]]; then
        failures=$((failures + 1))
        failure_notes+=("unreachable rule SET differs from constraints.allowed_unreachable_rules: observed=${observed_unreachable_rules_json} allowed=${allowed_unreachable_rules_json} (a listed rule that is no longer observed is GOOD NEWS and an invitation to re-derive the contract, not a defect to suppress)")
    elif ! jq -e --arg reason "$required_unreachable_rule_reason" \
              'all(.unreachable_rule_debt[]?; .reachable == false and .reason == $reason)' \
              "$gap_json" >/dev/null; then
        failures=$((failures + 1))
        failure_notes+=("an allowed unreachable rule carries a reason other than '${required_unreachable_rule_reason}'")
    fi
elif (( unreachable_rules > max_unreachable_rules )); then
    failures=$((failures + 1))
    failure_notes+=("unreachable_rules=${unreachable_rules} > max_unreachable_rules=${max_unreachable_rules}")
fi
if (( unreachable_branches > max_unreachable_branches )); then
    failures=$((failures + 1))
    failure_notes+=("unreachable_branches=${unreachable_branches} > max_unreachable_branches=${max_unreachable_branches}")
fi
if (( reachable_rules < min_reachable_rules )); then
    failures=$((failures + 1))
    failure_notes+=("reachable_rules=${reachable_rules} < min_reachable_rules=${min_reachable_rules}")
fi

jq -n \
  --arg grammar_name "$grammar_name" \
  --arg grammar_file "$grammar_file" \
  --arg grammar_json "$grammar_json" \
  --arg parser_out "$parser_out" \
  --arg entry_rule "$entry_rule" \
  --argjson contract_version "$contract_version" \
  --argjson defined_rule_count "$defined_rule_count" \
  --argjson unique_rule_name_count "$unique_rule_name_count" \
  --argjson rule_reference_count "$rule_reference_count" \
  --argjson unique_rule_reference_count "$unique_rule_reference_count" \
  --argjson unresolved_rule_reference_count "$unresolved_rule_reference_count" \
  --argjson entry_rule_defined "$entry_rule_defined" \
  --argjson reachable_rules "$reachable_rules" \
  --argjson unreachable_rules "$unreachable_rules" \
  --argjson reachable_branches "$reachable_branches" \
  --argjson unreachable_branches "$unreachable_branches" \
  --argjson target_debt_count "$target_debt_count" \
  --argjson failures "$failures" \
  --argjson constraints "$(jq -c '.constraints' "$CONTRACT_FILE")" \
  '
  {
    grammar_name: $grammar_name,
    grammar_file: $grammar_file,
    grammar_raw_ast_json: $grammar_json,
    generated_parser_file: $parser_out,
    entry_rule: $entry_rule,
    contract_version: $contract_version,
    metrics: {
      defined_rule_count: $defined_rule_count,
      unique_rule_name_count: $unique_rule_name_count,
      rule_reference_count: $rule_reference_count,
      unique_rule_reference_count: $unique_rule_reference_count,
      unresolved_rule_reference_count: $unresolved_rule_reference_count,
      entry_rule_defined: ($entry_rule_defined == 1),
      reachable_rules: $reachable_rules,
      unreachable_rules: $unreachable_rules,
      reachable_branches: $reachable_branches,
      unreachable_branches: $unreachable_branches,
      target_debt_count: $target_debt_count
    },
    constraints: $constraints,
    status: (if $failures == 0 then "pass" else "fail" end),
    failure_count: $failures
  }' >"$SUMMARY_JSON"

{
    echo "PGEN ${grammar_name} Syntax Closure Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo "grammar_name: $grammar_name"
    echo "grammar_file: $grammar_file"
    echo "grammar_raw_ast_json: $grammar_json"
    echo "generated_parser_file: $parser_out"
    echo "entry_rule: $entry_rule"
    echo
    echo "Metrics:"
    echo "  defined_rule_count: $defined_rule_count"
    echo "  unique_rule_name_count: $unique_rule_name_count"
    echo "  rule_reference_count: $rule_reference_count"
    echo "  unique_rule_reference_count: $unique_rule_reference_count"
    echo "  unresolved_rule_reference_count: $unresolved_rule_reference_count"
    echo "  entry_rule_defined: $entry_rule_defined"
    echo "  reachable_rules: $reachable_rules"
    echo "  unreachable_rules: $unreachable_rules"
    echo "  reachable_branches: $reachable_branches"
    echo "  unreachable_branches: $unreachable_branches"
    echo "  target_debt_count: $target_debt_count"
    echo
    echo "Artifacts:"
    echo "  summary_json: $SUMMARY_JSON"
    echo "  unresolved_refs: $unresolved_refs_file"
    echo "  parser_out: $parser_out"
    echo "  coverage_json: $coverage_json"
    echo "  gap_json: $gap_json"
    echo "  gap_txt: $gap_txt"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if (( failures > 0 )); then
    echo "❌ ${grammar_name} syntax closure gate failed with ${failures} violation(s):" >&2
    for note in "${failure_notes[@]}"; do
        echo "  - $note" >&2
    done
    # ⛔⛔ STALE **AND** RED IS THE ONE CELL THAT GENUINELY NEEDS A PERSON, and saying so is the
    # entire value of carrying an identity. Without it this exit reads "the tree regressed", which
    # is exactly the wrong diagnosis when the baseline is the stale half — the shape that left
    # this contract wrong for 69 grammar revisions.
    if (( identity_stale == 1 )); then
        {
            echo ""
            echo "  ⛔ AND THIS BASELINE IS STALE: a declared input moved since it was confirmed."
            echo "     So the failure above is AMBIGUOUS — it may be a real regression, or the"
            echo "     contract may simply no longer describe this tree. Adjudicate it; do NOT"
            echo "     re-stamp to make it green, because a stamp asserts the numbers were"
            echo "     RE-DERIVED and matched, which is precisely what just did not happen."
        } >&2
    fi
    exit 1
fi

# ⭐⭐ STALE + GREEN => THIS RUN IS THE CONFIRMING RUN. Every constraint was evaluated against the
# current tree and every one held, so the baseline was stale and still correct — a DERIVED fact
# needing no human judgement. Re-stamping here is what removes the adoption friction: no escape
# variable, no second run, no remembered command.
# ⛔ The guard that keeps it honest: this is reached ONLY after `failures == 0`, and every stage
# above exits non-zero rather than skipping, so a vacuous green cannot arrive here.
if (( identity_stale == 1 )); then
    echo "==> baseline_identity_restamp"
    if "$ROOT_DIR/scripts/check_baseline_identity.sh" --stamp "${CONTRACT_FILE#"$ROOT_DIR/"}" \
        --confirmed-by "auto re-stamped by a GREEN ${grammar_name} syntax closure gate run: \
defined_rule_count=${defined_rule_count} reachable_rules=${reachable_rules} \
unreachable_rules=${unreachable_rules} unreachable_branches=${unreachable_branches} \
unresolved_rule_reference_count=${unresolved_rule_reference_count} \
entry_rule=${entry_rule} stimuli_seed=${stimuli_seed}"; then
        echo "    the baseline was STALE and every constraint still held, so this run re-stamped it"
    else
        echo "    ⛔ the re-stamp FAILED — the gate's verdict stands, but the baseline is still stale" >&2
        exit 1
    fi
fi

echo "✅ ${grammar_name} syntax closure gate passed."
