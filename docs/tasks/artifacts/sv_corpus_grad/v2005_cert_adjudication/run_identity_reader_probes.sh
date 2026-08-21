#!/usr/bin/env bash
# run_identity_reader_probes.sh — SV-CORPUS-GRAD.13c.2x.7 (a)/(d)
#
# PROVES THE READER ACTUALLY BITES. `BASELINE-IDENTITY`'s founding measurement (SV-CORPUS-GRAD.13i)
# is that SIX oracles already carried a self-describing identity block, exactly ONE was gate-checked,
# and FOUR were measurably stale — so a block with no consumer passes every check that only asks
# whether it exists. Adding a block to `verilog_2005_conformance_contract_v0.json` without OBSERVING
# its reader refuse would reproduce that defect, not repair it.
#
# ⭐ EVERY ARM IS CHEAP BY DESIGN, AND THAT IS THE POINT OF WHERE THE READER SITS. The gate's
# `build_release_parseability_probe` stage costs ~20 minutes cold; the identity check runs BEFORE it,
# so a refusal arm returns in about a second. An arm that took 20 minutes to observe would not be
# re-run, and a control nobody re-runs is the `.13i` shape one level up.
#
# Each arm asserts BOTH the exit code AND a substring of the message (CI-PARITY-GATE-ROT.4's rule:
# pass/fail alone hides an arm that reaches the right verdict for the wrong reason).
#
#   bash docs/tasks/artifacts/sv_corpus_grad/v2005_cert_adjudication/run_identity_reader_probes.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

GATE="$ROOT/rust/scripts/verilog_2005_conformance_gate.sh"
LIVE="$ROOT/rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json"
WORK="$ROOT/rust/target/v2005_adjudication/identity_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0
arm() {
    local label="$1" want_rc="$2" want_substr="$3" contract="$4"
    local out rc
    out="$(PGEN_VERILOG_2005_CONFORMANCE_CONTRACT_FILE="$contract" bash "$GATE" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" && "$out" == *"$want_substr"* ]]; then
        printf '  ✅ %-52s exit=%s, message matched\n' "$label" "$rc"; pass=$((pass + 1))
    else
        printf '  ⛔ %-52s exit=%s (want %s)\n' "$label" "$rc" "$want_rc"
        printf '     wanted substring: %s\n' "$want_substr"
        printf '%s\n' "$out" | tail -n 8 | sed 's/^/     | /'
        fail=$((fail + 1))
    fi
}

echo "=============================================================================="
echo "verilog_2005_conformance_gate — BASELINE-IDENTITY reader arms"
echo "=============================================================================="

# RED-1 — NO BLOCK AT ALL. This is the state the contract was in for its entire life, through two
# hand-rebaselines and 22 grammar revisions: the gate could say "something is wrong" and never
# "your baseline is 22 revisions old".
python3 -c "
import json,sys
d=json.load(open('$LIVE')); d.pop('identity',None)
json.dump(d,open('$WORK/no_block.json','w'),indent=2)
"
arm "RED-1 no identity block refuses (exit 2)" 2 "REFUSING TO MEASURE" "$WORK/no_block.json"

# RED-2 — UNCONFIRMED expectations refuse BEFORE the ~20 min build. A person has already recorded
# that the numbers do not describe the tree, so measuring teaches nothing.
# ⛔ THIS ARM MUTATES THE LIVE BLOCK RATHER THAN HAND-BUILDING ONE, DELIBERATELY, AND THAT ORDERS IT:
# it can only pass once the contract HAS an identity block to mutate. Run before the block existed it
# fails — the constructed dict has no `_verifier`, so the enforcer rejects it as MALFORMED instead of
# as UNCONFIRMED. That was observed, not assumed (SV-CORPUS-GRAD.13c.2x.7, first probe run), and it is
# a property worth keeping: a block missing the field that tells a reader what re-derives it is not a
# lenient case, it is a refusal. Hand-building a synthetic block here would have hidden that.
python3 -c "
import json
d=json.load(open('$LIVE'))
i=dict(d.get('identity') or {})
i['expectations']='unconfirmed'
i['unconfirmed_reason']='probe arm: deliberately marked unconfirmed to observe the refusal'
i['owner_leaf']='SV-CORPUS-GRAD.13c.2x.7'
i.pop('confirmed_by',None)
d['identity']=i
json.dump(d,open('$WORK/unconfirmed.json','w'),indent=2)
"
arm "RED-2 unconfirmed expectations refuse (exit 2)" 2 "are UNCONFIRMED" "$WORK/unconfirmed.json"

# RED-3 — A MALFORMED block refuses rather than falling back to 'no block, carry on'. A mis-typed
# block that degraded to "unchecked" would restore the exact blindness this doctrine removes.
python3 -c "
import json
d=json.load(open('$LIVE'))
d['identity']={'inputs':'not-a-map'}
json.dump(d,open('$WORK/malformed.json','w'),indent=2)
"
arm "RED-3 malformed block refuses (exit 2)" 2 "REFUSING TO MEASURE" "$WORK/malformed.json"

echo "------------------------------------------------------------------------------"
printf 'verilog_2005 identity reader probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
