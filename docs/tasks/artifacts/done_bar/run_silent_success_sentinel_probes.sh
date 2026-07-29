#!/usr/bin/env bash
# run_silent_success_sentinel_probes.sh — probe arms for the SILENT-SUCCESS
# SENTINEL gate (DONE-BAR.5c).
#
# Every arm asserts BOTH the exit code AND a substring of the message it expects.
# ⭐ Exit code alone is not enough: CI-PARITY-GATE-ROT.4 measured 8 RED arms
# reaching the FAIL verdict they wanted for the WRONG REASON, and a driver
# comparing only PASS/FAIL would have reported them green over an audit that
# evaluated none of them.
#
# Arms:
#   GREEN-1  the untouched tree passes (exit 0)
#   CTRL-1   ⭐ THE ARM THAT MATTERS — the POSITIVE calibration control reads 1.
#            If a LATENT site reads 0 the detector is BLIND and every clean
#            sweep is worthless. A zero here must be indistinguishable from a
#            broken detector, and it is: the gate refuses.
#   CTRL-2   the artifact roster is DERIVED — generated/ebnf.rs (a shipped
#            parser WITHOUT the `_parser.rs` suffix) is counted. The suffix glob
#            silently skipped its 123 arms on the gate's first run.
#   RED-1    a codegen placeholder injected into a shipped artifact ⇒ FAIL(1)
#   RED-2    a REACHED sentinel on a family's own surface ⇒ FAIL(1), family named
#   RED-3    a calibration expectation that no longer holds ⇒ MISCALIBRATED(2),
#            and NO sweep verdict is offered
#   RED-4    generated/ artifacts absent ⇒ REFUSE(2), never a green over an
#            empty room (the GENERATED-LINT-CORRECTNESS.3 vacuity trap)
#   RED-5    a family whose generation window yields nothing ⇒ REFUSE(2) for
#            that family, never a green zero
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
GATE="$ROOT_DIR/rust/scripts/silent_success_sentinel_gate.sh"
CONTRACT="$ROOT_DIR/rust/test_data/grammar_quality/silent_success_sentinel_contract_v0.json"
TMP="$ROOT_DIR/rust/target/silent_success_sentinel_probes"
rm -rf "$TMP"; mkdir -p "$TMP"

pass=0; fail=0

run_arm() {
    local name="$1" want_code="$2" want_text="$3"; shift 3
    local out code
    out="$("$@" 2>&1)"; code=$?
    if [[ "$code" == "$want_code" ]] && grep -qF -- "$want_text" <<<"$out"; then
        echo "  ✅ $name (exit $code, matched: ${want_text:0:60})"
        pass=$((pass + 1))
    else
        echo "  ❌ $name — wanted exit $want_code + '${want_text:0:60}', got exit $code"
        echo "$out" | tail -12 | sed 's/^/       /'
        fail=$((fail + 1))
    fi
}

# Build a FAKE ROOT holding the gate at its real relative path. The gate derives
# ROOT_DIR from ${BASH_SOURCE[0]}, NOT from the cwd (correct — the repo-root
# relative-path policy means the tree can be relocated), so an arm that only
# `cd`s elsewhere still measures the REAL generated/. This probe learned that
# the hard way: RED-4's first cut passed exit 0 over the live tree.
make_fake_root() {
    local dest="$1"; shift
    mkdir -p "$dest/rust/scripts/lib" "$dest/rust/test_data/grammar_quality" "$dest/generated"
    cp "$GATE" "$dest/rust/scripts/"
    cp "$ROOT_DIR/rust/scripts/lib/silent_success_sentinel_sweep.py" "$dest/rust/scripts/lib/"
    mkdir -p "$dest/rust/target/debug"
    for b in ast_pipeline parseability_probe; do
        ln -sf "$ROOT_DIR/rust/target/debug/$b" "$dest/rust/target/debug/$b"
    done
    echo "$dest/rust/scripts/silent_success_sentinel_gate.sh"
}

echo "=== SILENT-SUCCESS SENTINEL probe arms ==="

# ---------------------------------------------------------------- GREEN-1
PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/green1" \
run_arm "GREEN-1 untouched tree passes" 0 \
    "no reachable silent-success path" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/green1" bash "$GATE"

# ----------------------------------------------------------------- CTRL-1
# The positive control must report exactly 1. Read it from the gate's own
# report so the arm tests what the gate believes, not a re-implementation.
cal2="$(jq -r '.report.calibration[] | select(.id=="CAL-2-latent-is-visible") | .observed_sentinels' \
        "$TMP/green1/summary.json" 2>/dev/null)"
if [[ "$cal2" == "1" ]]; then
    echo "  ✅ CTRL-1 positive control sees a real sentinel (CAL-2 observed=1)"
    pass=$((pass + 1))
else
    echo "  ❌ CTRL-1 positive control observed='$cal2', expected 1 — DETECTOR IS BLIND"
    fail=$((fail + 1))
fi

# ----------------------------------------------------------------- CTRL-2
scanned="$(jq -r '.report.static.parsers_scanned' "$TMP/green1/summary.json" 2>/dev/null)"
on_disk="$(ls "$ROOT_DIR"/generated/*.rs 2>/dev/null | wc -l | tr -d ' ')"
if [[ "$scanned" == "$on_disk" ]] && (( on_disk > 0 )); then
    echo "  ✅ CTRL-2 artifact roster is DERIVED (scanned $scanned == $on_disk on disk)"
    pass=$((pass + 1))
else
    echo "  ❌ CTRL-2 scanned=$scanned but $on_disk generated/*.rs on disk — roster not derived"
    fail=$((fail + 1))
fi

# ----------------------------------------------------------------- CTRL-3
# The generation input must be the raw-AST JSON, never the .ebnf. Feeding the
# .ebnf needs ast_pipeline built with --features ebnf_dual_run, which the
# standard build -- including the one `regenerate_generated_parsers` leaves
# behind -- does NOT enable, so an .ebnf-fed gate REFUSES right after a
# regeneration. Measured: that is exactly how this gate first broke.
bad_inputs="$(jq -r '[.families[] | select((.gen_input // .ebnf) | startswith("generated/") | not) | .grammar] | join(",")' "$CONTRACT")"
if [[ -z "$bad_inputs" ]]; then
    echo "  ✅ CTRL-3 every family generates from generated/*.json (no ebnf_dual_run dependency)"
    pass=$((pass + 1))
else
    echo "  ❌ CTRL-3 these families would read a .ebnf and REFUSE after a regeneration: $bad_inputs"
    fail=$((fail + 1))
fi

# ------------------------------------------------------------------ RED-1
# A codegen placeholder appearing in a shipped artifact. Injected into a COPY of
# generated/ inside a fake root, so the real tree is never mutated AND the real
# gate (not a re-implementation) produces the verdict.
RED1_ROOT="$TMP/red1_root"
RED1_GATE="$(make_fake_root "$RED1_ROOT")"
cp "$ROOT_DIR"/generated/*.rs "$RED1_ROOT/generated/"
printf 'ParseContent::Terminal("<property_access>")\n' >>"$RED1_ROOT/generated/json_parser.rs"
python3 - "$CONTRACT" "$TMP/red1_contract.json" <<'PY_RED1'
import json, sys
c = json.load(open(sys.argv[1]))
c["families"] = []                      # isolate the STATIC arm
c["calibration"]["facts"] = []
json.dump(c, open(sys.argv[2], "w"))
PY_RED1
run_arm "RED-1 codegen placeholder in a shipped artifact ⇒ FAIL" 1 \
    "codegen placeholder <property_access> present in" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/red1" \
        PGEN_SILENT_SUCCESS_CONTRACT="$TMP/red1_contract.json" bash "$RED1_GATE"

# ------------------------------------------------------------------ RED-2
# A REACHED sentinel on a family's own surface. Rather than corrupt a parser,
# the contract's sentinel ROSTER is mutated to include a token every AST dump
# legitimately contains — so a normal sweep must now REACH it. This exercises
# the whole dynamic path end-to-end: detect -> aggregate -> verdict -> message.
python3 - "$CONTRACT" "$TMP/red2_contract.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))
c["sweep"]["sample_count"] = 2
c["families"] = [f for f in c["families"] if f["grammar"] == "json"]
c["calibration"]["facts"] = []   # isolate the DYNAMIC arm: calibration runs
                                 # FIRST by design, and this roster mutation
                                 # would legitimately trip it
c["sentinel_roster"]["runtime_fallback"]["literals"].append('"content"')
json.dump(c, open(sys.argv[2], "w"))
PY
run_arm "RED-2 a REACHED sentinel ⇒ FAIL naming the family" 1 \
    "json sample #" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/red2" \
        PGEN_SILENT_SUCCESS_CONTRACT="$TMP/red2_contract.json" bash "$GATE"

# ------------------------------------------------------------------ RED-3
python3 - "$CONTRACT" "$TMP/red3_contract.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))
c["families"] = []
for f in c["calibration"]["facts"]:
    if f["id"] == "CAL-2-latent-is-visible":
        f["expected_sentinels"] = 99          # a fact that no longer holds
json.dump(c, open(sys.argv[2], "w"))
PY
run_arm "RED-3 calibration drift ⇒ MISCALIBRATED, no verdict" 2 \
    "MISCALIBRATED" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/red3" \
        PGEN_SILENT_SUCCESS_CONTRACT="$TMP/red3_contract.json" bash "$GATE"

# ------------------------------------------------------------------ RED-4
EMPTY_ROOT="$TMP/red4_root"
EMPTY_GATE="$(make_fake_root "$EMPTY_ROOT")"      # generated/ deliberately empty
run_arm "RED-4 no generated/ artifacts ⇒ REFUSE, never a green empty room" 2 \
    "a skip is never a pass" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/red4" bash "$EMPTY_GATE"

# ------------------------------------------------------------------ RED-5
python3 - "$CONTRACT" "$TMP/red5_contract.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))
c["calibration"]["facts"] = []
# rtl_const_expr at the DEFAULT window generates nothing (its ~16-level
# precedence cascade exceeds depth 24). The gate must REFUSE for it, not
# report a green zero.
c["families"] = [dict(f, max_depth=None) for f in c["families"]
                 if f["grammar"] == "rtl_const_expr"]
c["families"][0].pop("max_depth")
json.dump(c, open(sys.argv[2], "w"))
PY
run_arm "RED-5 a family that yields no samples ⇒ REFUSE, not a green zero" 2 \
    "could not be measured" \
    env PGEN_SILENT_SUCCESS_STATE_DIR="$TMP/red5" \
        PGEN_SILENT_SUCCESS_CONTRACT="$TMP/red5_contract.json" bash "$GATE"

echo
echo "=== probes: $pass passed / $fail failed ==="
[[ "$fail" == "0" ]]
