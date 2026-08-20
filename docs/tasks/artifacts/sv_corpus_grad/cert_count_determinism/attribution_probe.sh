#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x.1 (d) — ADVERSARIAL PROBE for the drift ATTRIBUTION helper.
#
# ⛔ A mechanism that only ever says one thing is indistinguishable from a hole
# ([[a-check-whose-inputs-all-pass-has-not-been-tested]]). This helper exists to tell two causes
# apart, so BOTH verdicts must be observed, and the input that decides between them must be shown
# to be load-bearing rather than decorative.
#
# ⛔ THE ROOT IS FOUND BY WALKING UP TO A SENTINEL, never by counting `..` — three probes in this
# repository have shipped with the wrong depth.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while [[ "$ROOT" != "/" && ! ( -f "$ROOT/grammars/systemverilog.ebnf" && -f "$ROOT/scripts/check_doctrines.sh" ) ]]; do
    ROOT="$(dirname "$ROOT")"
done
[[ -f "$ROOT/scripts/check_doctrines.sh" ]] || { echo "probe: no repo root above $(dirname "${BASH_SOURCE[0]}")" >&2; exit 2; }
cd "$ROOT" || exit 2

LIB="$ROOT/rust/scripts/lib/drift_attribution.sh"
GATE="$ROOT/rust/scripts/sv_cert_recognized_union_gate.sh"
[[ -f "$LIB"  ]] || { echo "probe: helper missing: $LIB" >&2; exit 2; }
[[ -f "$GATE" ]] || { echo "probe: gate missing: $GATE" >&2; exit 2; }
# shellcheck source=/dev/null
source "$LIB"

PASS=0; FAIL=0
arm() {  # $1=name  $2=expect-substring  $3=expect-rc  rest=args to the helper
    local name="$1" want="$2" want_rc="$3"; shift 3
    local out rc
    out="$(attribute_signature_drift "$@" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" && "$out" == *"$want"* ]]; then
        printf '  ✅ %-56s rc=%s\n' "$name" "$rc"; PASS=$((PASS+1))
    else
        printf '  ❌ %-56s rc=%s (wanted rc=%s + %q)\n' "$name" "$rc" "$want_rc" "$want"
        printf '     got: %s\n' "${out:0:200}"; FAIL=$((FAIL+1))
    fi
}
static() {  # $1=name  $2=expect-substring  $3=file
    local name="$1" want="$2" f="$3"
    if grep -qF -- "$want" "$f"; then printf '  ✅ %-56s\n' "$name"; PASS=$((PASS+1))
    else printf '  ❌ %-56s (no %q in %s)\n' "$name" "$want" "${f#"$ROOT/"}"; FAIL=$((FAIL+1)); fi
}

SIG_A="1434|8|1361|65|0|1434|8|1380|54|[]"
SIG_B="1433|8|1361|64|0|1433|8|1380|53|[]"
SHA_A="875ecab5de723631a9ad7707138628b42d551027223d0ae02cf8dbf7f2eb86e4"
SHA_B="bd9367dc1d79b6f2531a3975cb5c2449c451817f97c1a92fbcb18200fd14c9b4"

echo "── the two verdicts this helper exists to tell apart ──"
arm "1  input MOVED  -> names the input, not the tool" \
    "THE INPUT CHANGED UNDER THIS RUN" 0 7 0 "$SHA_B" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A"
arm "2  input HELD   -> a real tool-nondeterminism finding" \
    "INPUT HELD BYTE-IDENTICAL" 0 7 0 "$SHA_A" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A"

echo
echo "── arm 3 is what makes the digest LOAD-BEARING rather than decoration ──"
# ⭐ Byte-for-byte arm 1 with ONLY the second digest changed to match the first. The verdict MUST
# flip. Without this, the helper could ignore its digests entirely and still score 2/2 above.
arm "3  arm 1 with ONLY the sha equalised -> verdict FLIPS" \
    "INPUT HELD BYTE-IDENTICAL" 0 7 0 "$SHA_A" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A"

echo
echo "── refusals: it must not invent an attribution it cannot support ──"
arm "4  a digest was never recorded -> UNATTRIBUTABLE" \
    "UNATTRIBUTABLE" 0 7 0 "" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A"
arm "5  the OTHER digest missing    -> UNATTRIBUTABLE" \
    "UNATTRIBUTABLE" 0 7 0 "$SHA_A" "" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A"
arm "6  wrong argument count        -> refuses, rc 2" \
    "refusing to compose a drift message" 2 7 0 "$SHA_A"

echo
echo "── the message must carry the evidence a reader needs to act ──"
out_moved="$(attribute_signature_drift 7 0 "$SHA_B" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A")"
for want in "${SHA_A:0:16}" "${SHA_B:0:16}" "grammars/systemverilog.ebnf" "quiescent tree"; do
    if [[ "$out_moved" == *"$want"* ]]; then printf '  ✅ %-56s\n' "moved-message carries ${want:0:34}"; PASS=$((PASS+1))
    else printf '  ❌ %-56s\n' "moved-message MISSING ${want:0:34}"; FAIL=$((FAIL+1)); fi
done
out_held="$(attribute_signature_drift 7 0 "$SHA_A" "$SHA_A" "grammars/systemverilog.ebnf" "$SIG_B" "$SIG_A")"
for want in "DE-CONFOUND" "cert_count_determinism/probe.sh"; do
    if [[ "$out_held" == *"$want"* ]]; then printf '  ✅ %-56s\n' "held-message points at ${want:0:34}"; PASS=$((PASS+1))
    else printf '  ❌ %-56s\n' "held-message MISSING ${want:0:34}"; FAIL=$((FAIL+1)); fi
done

echo
echo "── the consuming gate is actually wired to it (one implementation, not two) ──"
static "gate SOURCEs the shared helper"        'source "$ROOT_DIR/rust/scripts/lib/drift_attribution.sh"' "$GATE"
static "gate CALLS it on drift"                'attribute_signature_drift "$seed" "$first_seed_id"'       "$GATE"
static "gate digests the grammar PER SEED"     'seed_input_sha="$(shasum -a 256 "$GRAMMAR_FILE"'          "$GATE"
static "gate RECORDS the digest per seed"      '--arg grammar_sha256 "$seed_input_sha"'                   "$GATE"
# ⚠️ The phrase is SPLIT across a shell line-continuation in the helper source, so this asserts on
# the fragment that actually exists on one line. The probe's first run failed here — on its own
# assertion, not on the helper — which is the cheap version of the mistake it is guarding against.
static "the message text LIVES in the shared helper" 'THE INPUT CHANGED' "$LIB"
if grep -qF 'THE INPUT CHANGED UNDER THIS RUN' "$GATE"; then
    printf '  ❌ %-56s\n' "gate carries its OWN copy of the message"; FAIL=$((FAIL+1))
else
    printf '  ✅ %-56s\n' "gate carries no second copy of the message"; PASS=$((PASS+1))
fi

echo
echo "── the INDEPENDENT census must see it, through its own unmodified predicate ──"
# ⛔ This is the arm that stops the fix being self-graded. `gate_input_pin_census.sh` was written
# BEFORE this guard and is not modified by it; if the gate had hidden its digest behind a helper
# defined above the loop, the census would still read BLIND and the only honest options would have
# been to move the code or to tune the census — and tuning the measure to flatter the change is the
# defect, not the fix.
census_out="$(bash "$ROOT/docs/tasks/artifacts/sv_corpus_grad/cert_count_determinism/gate_input_pin_census.sh" 2>&1)"
if [[ "$census_out" == *"sv_cert_recognized_union_gate.sh"*"pins"* ]]; then
    printf '  ✅ %-56s\n' "census reports the SV cert gate as pinning"; PASS=$((PASS+1))
else
    printf '  ❌ %-56s\n' "census still reports the SV cert gate BLIND"; FAIL=$((FAIL+1))
fi
if [[ "$census_out" == *"blind_to_input_change=4"* ]]; then
    printf '  ✅ %-56s\n' "census headline moved 5 -> 4 blind"; PASS=$((PASS+1))
else
    printf '  ❌ %-56s\n' "census headline is not 4"; FAIL=$((FAIL+1))
fi

echo
echo "ATTRIBUTION-PROBE: passed=${PASS} failed=${FAIL}"
[[ "$FAIL" -eq 0 ]] || exit 1
