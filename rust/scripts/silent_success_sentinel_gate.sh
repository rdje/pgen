#!/usr/bin/env bash
# silent_success_sentinel_gate.sh — the SILENT-SUCCESS SENTINEL gate (DONE-BAR.5c).
#
# A silent success is a parse that returns Ok with ZERO diagnostics while handing
# the consumer a PLACEHOLDER node instead of the shape the return annotation
# promised. Every "did it parse?" gate is GREEN on such a parse — which is exactly
# why this instrument has to exist separately: the defect is invisible to
# acceptance testing by construction.
#
# Two arms, both pinned by
# rust/test_data/grammar_quality/silent_success_sentinel_contract_v0.json:
#
#   STATIC   the codegen-placeholder sentinels (<property_access> / <array_access>
#            / <last_extraction>) must be ABSENT from every shipped generated
#            parser. Measured 0 today; this arm locks that so it cannot drift in.
#   DYNAMIC  each family's own stimuli proof surface is generated under
#            --validate-parseability, parsed, and AST-dumped. ANY sentinel
#            actually reached is a defect: the consumer got a placeholder inside
#            a SUCCESSFUL parse.
#
# ⛔ THE STATIC ARM ALONE WOULD BE VACUOUS. DONE-BAR.5c was chartered against the
# codegen half only, and those literals occur 0 times across all 11 shipped
# parsers — a gate asserting just their absence passes over an untouched tree and
# can never fail. The SHIPPED sentinel surface is the RUNTIME half (3702 fallback
# arms in 10 of 11 parsers) with a proven consumer-visible corruption history
# (ledger SV-0014..SV-0020, SVPP-0001, RTL-FE-0002, VHDL-0001, RTL-CE-0001).
#
# ⭐ CALIBRATION IS PART OF THE CHECK. A detector with no POSITIVE control cannot
# tell a clean sweep from a blind one — a zero reads as a pass either way. Three
# pinned ground-truth facts must reproduce (a fixed ledger repro at 0, a LATENT
# site at 1 under --entry-rule isolation, and that same site at 0 from the
# canonical entry) or the gate prints MISCALIBRATED and exits nonzero WITHOUT
# offering a sweep verdict.
#
# ⚠️ HONEST BOUND, stated in the gate's own output: the dynamic arm proves
# "NOT REACHED by <sample_count> validated samples at the pinned seed", which is
# NOT a proof of unreachability. Reachability is entry-relative.
#
# Exit codes:
#   0  every arm clean
#   1  a sentinel was REACHED, or a codegen placeholder appeared in an artifact
#   2  REFUSE — the gate could not see what it is meant to judge (missing
#      artifacts/binaries, zero samples, extractor suspect, or MISCALIBRATED).
#      A skip is never a pass.
#
# Env overrides:
#   PGEN_SILENT_SUCCESS_CONTRACT   — alternate contract path (testability hook)
#   PGEN_SILENT_SUCCESS_STATE_DIR  — alternate state dir
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

CONTRACT="${PGEN_SILENT_SUCCESS_CONTRACT:-$RUST_DIR/test_data/grammar_quality/silent_success_sentinel_contract_v0.json}"
STATE_DIR="${PGEN_SILENT_SUCCESS_STATE_DIR:-$RUST_DIR/target/silent_success_sentinel_gate}"
WORK_DIR="$STATE_DIR/work"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
REPORT_JSON="$STATE_DIR/sweep.json"

SWEEP_PY="$RUST_DIR/scripts/lib/silent_success_sentinel_sweep.py"
AST_PIPELINE="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE="$RUST_DIR/target/debug/parseability_probe"

GATE_NAME="silent_success_sentinel_gate"

refuse() {
    mkdir -p "$STATE_DIR"
    {
        echo "$GATE_NAME: REFUSE"
        echo "reason: $1"
        echo "⛔ a skip is never a pass — this gate exits 2 rather than reporting a green it cannot justify"
    } | tee "$SUMMARY_TXT"
    jq -n --arg gate "$GATE_NAME" --arg reason "$1" \
        '{gate:$gate,status:"refuse",exit:2,reason:$reason}' >"$SUMMARY_JSON"
    exit 2
}

for tool in jq python3; do
    command -v "$tool" >/dev/null 2>&1 || refuse "required tool '$tool' not in PATH"
done

[[ -f "$CONTRACT" ]]  || refuse "contract not found: $CONTRACT"
[[ -f "$SWEEP_PY" ]]  || refuse "sweep helper not found: $SWEEP_PY"

# generated/ is UNTRACKED. A clean checkout would sweep an empty room, and a
# naive gate would exit 0 having examined nothing — the GENERATED-LINT-CORRECTNESS.3
# vacuity trap. Refuse instead.
# NB: every shipped .rs artifact, not `*_parser.rs` — generated/ebnf.rs is a
# shipped parser without that suffix (see the sweep helper's static arm).
shopt -s nullglob
generated_parsers=("$ROOT_DIR"/generated/*.rs)
shopt -u nullglob
(( ${#generated_parsers[@]} > 0 )) || refuse \
    "no generated/*_parser.rs artifacts — regenerate with 'make -C rust regenerate_generated_parsers'"

[[ -x "$AST_PIPELINE" ]] || refuse "missing binary $AST_PIPELINE (build with --features generated_parsers)"
[[ -x "$PARSE_PROBE"  ]] || refuse "missing binary $PARSE_PROBE (build with --features generated_parsers)"

mkdir -p "$STATE_DIR" "$WORK_DIR"

python3 "$SWEEP_PY" "$ROOT_DIR" "$CONTRACT" "$WORK_DIR" >"$REPORT_JSON" \
    || refuse "sweep helper failed; see $REPORT_JSON"

# ------------------------------------------------------------------ verdicts

static_status="$(jq -r '.static.status' "$REPORT_JSON")"
[[ "$static_status" == "OK" ]] || refuse \
    "static arm: $static_status — $(jq -r '.static.detail // ""' "$REPORT_JSON")"

# CALIBRATION FIRST — before any sweep result is believed.
mapfile -t cal_bad < <(jq -r '.calibration[] | select(.match != true) | .id' "$REPORT_JSON")
if (( ${#cal_bad[@]} > 0 )); then
    mkdir -p "$STATE_DIR"
    {
        echo "$GATE_NAME: MISCALIBRATED"
        echo "the instrument did not reproduce pinned ground truth, so NO sweep verdict is offered:"
        jq -r '.calibration[] | select(.match != true)
               | "  \(.id): observed \(.observed_sentinels) sentinels, expected \(.expected_sentinels) (parsed=\(.parsed)) \(.detail)"' \
            "$REPORT_JSON"
        echo "⭐ CAL-2 is the arm that matters: if a LATENT site reads 0, the detector is BLIND and every clean sweep is worthless."
    } | tee "$SUMMARY_TXT"
    jq -n --arg gate "$GATE_NAME" --argjson cal "$(jq '.calibration' "$REPORT_JSON")" \
        '{gate:$gate,status:"miscalibrated",exit:2,calibration:$cal}' >"$SUMMARY_JSON"
    exit 2
fi

# A family the sweep could not measure REFUSES — it never reports a green zero.
mapfile -t fam_refused < <(jq -r '.families[] | select(.status | startswith("REFUSE")) | "\(.grammar): \(.status) \(.detail // "")"' "$REPORT_JSON")
if (( ${#fam_refused[@]} > 0 )); then
    mkdir -p "$STATE_DIR"
    {
        echo "$GATE_NAME: REFUSE"
        echo "these families could not be measured, so no verdict is offered for them:"
        printf '  %s\n' "${fam_refused[@]}"
    } | tee "$SUMMARY_TXT"
    jq -n --arg gate "$GATE_NAME" --argjson fams "$(jq '[.families[] | select(.status|startswith("REFUSE"))]' "$REPORT_JSON")" \
        '{gate:$gate,status:"refuse",exit:2,families:$fams}' >"$SUMMARY_JSON"
    exit 2
fi

codegen_violations="$(jq '[.static.codegen_placeholder_violations | to_entries[]] | length' "$REPORT_JSON")"
reached_total="$(jq '[.families[].sentinel_totals // {} | to_entries[] | .value] | add // 0' "$REPORT_JSON")"
reached_families="$(jq '[.families[] | select(.sentinel_samples > 0)] | length' "$REPORT_JSON")"

{
    echo "$GATE_NAME"
    echo "contract_version: $(jq -r '.contract_version' "$REPORT_JSON")"
    echo
    echo "STATIC arm — codegen placeholders absent from shipped artifacts:"
    echo "  parsers scanned:               $(jq -r '.static.parsers_scanned' "$REPORT_JSON")"
    echo "  codegen-placeholder violations: $codegen_violations"
    echo "  runtime fallback arms PRESENT (not a defect on its own — reaching one is):"
    jq -r '.static.runtime_fallback_arms_present | to_entries[] | "    \(.key): \(.value)"' "$REPORT_JSON"
    echo
    echo "CALIBRATION arm — pinned ground truth reproduced:"
    jq -r '.calibration[] | "  \(.id): observed \(.observed_sentinels) == expected \(.expected_sentinels) ✅"' "$REPORT_JSON"
    echo
    echo "DYNAMIC arm — sentinels REACHED on each family's own stimuli surface"
    echo "  (samples=$(jq -r '.sample_count' "$REPORT_JSON") seed=$(jq -r '.seed' "$REPORT_JSON") per family):"
    jq -r '.families[] | "  \(.grammar): samples=\(.samples) parsed=\(.parsed_ok) reached=\(.sentinel_samples) sentinels=\([.sentinel_totals|to_entries[]|.value]|add // 0)"' "$REPORT_JSON"
    echo
    echo "families with a reached sentinel: $reached_families"
    echo "total sentinels reached:          $reached_total"
    echo
    echo "⚠️ HONEST BOUND: $(jq -r '.honest_limit' "$REPORT_JSON")"
} | tee "$SUMMARY_TXT"

if (( codegen_violations > 0 )) || (( reached_total > 0 )); then
    {
        echo
        echo "❌ $GATE_NAME FAILED — a parse can hand a consumer a placeholder inside a SUCCESSFUL parse."
        jq -r '.static.codegen_placeholder_violations | to_entries[]
               | "  codegen placeholder \(.key) present in: \(.value | to_entries | map("\(.key)×\(.value)") | join(", "))"' "$REPORT_JSON"
        jq -r '.families[] | select(.sentinel_samples > 0) | .grammar as $g
               | .examples[] | "  \($g) sample #\(.index) reached \(.found|to_entries|map("\(.key)×\(.value)")|join(", ")) on input: \(.sample)"' "$REPORT_JSON"
    } | tee -a "$SUMMARY_TXT"
    status="fail"; code=1
else
    echo "✅ $GATE_NAME passed — no reachable silent-success path on any family's proof surface." | tee -a "$SUMMARY_TXT"
    status="pass"; code=0
fi

jq -n \
    --arg gate "$GATE_NAME" \
    --arg status "$status" \
    --argjson exit_code "$code" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --argjson codegen_violations "$codegen_violations" \
    --argjson reached_total "$reached_total" \
    --argjson reached_families "$reached_families" \
    --argjson report "$(cat "$REPORT_JSON")" \
    '{gate:$gate,status:$status,exit:$exit_code,state_dir:$state_dir,
      summary_txt:$summary_txt,codegen_placeholder_violations:$codegen_violations,
      sentinels_reached_total:$reached_total,families_with_reached_sentinel:$reached_families,
      report:$report}' >"$SUMMARY_JSON"

exit "$code"
