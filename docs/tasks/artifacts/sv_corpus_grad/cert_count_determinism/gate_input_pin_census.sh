#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x.1 — WHICH GATES ASSERT DETERMINISM ACROSS RE-READS OF A MUTABLE INPUT,
# AND WHICH OF THEM PIN THAT INPUT?
#
# ⛔ THE SHAPE. A gate that re-reads a file once per seed and then asserts "every seed agreed"
# is making a claim about the TOOL. It can only mean that if the INPUT was the same each time —
# and nothing in these gates records what the input was. So a disagreement is reported as
# `signature drift vs seed 0`, which reads as *"the tool is nondeterministic"* when it is equally
# consistent with *"the file changed under me between iterations"*. A multi-seed run of the SV
# cert gate takes minutes; an operator applying or reverting an experimental arm in that window
# produces exactly this signal.
#
# ⭐ This census is mechanical and says so: it counts REFERENCES, so it sizes the population and
# does not adjudicate any individual gate. That is the same honest bound `.13c.2x`(d) put on its
# fifteen-baseline sweep.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while [[ "$ROOT" != "/" && ! ( -f "$ROOT/grammars/systemverilog.ebnf" && -f "$ROOT/scripts/check_doctrines.sh" ) ]]; do
    ROOT="$(dirname "$ROOT")"
done
[[ -f "$ROOT/scripts/check_doctrines.sh" ]] || { echo "census: no repo root found" >&2; exit 2; }
cd "$ROOT" || exit 2

# A gate ASSERTS cross-run determinism if it compares one run's result against another's.
DET_RE='first_seed_signature|signature drift|determinism tripwire|determinism"'
# It PINS its input if it records that input's identity anywhere in the comparison loop.
PIN_RE='shasum|sha256sum|--digest|check_baseline_identity'

printf '%-42s %-14s %-14s %s\n' GATE ASSERTS-DETERMINISM PINS-INPUT VERDICT
printf '%-42s %-14s %-14s %s\n' "$(printf '%.0s-' {1..42})" -------------- -------------- -------
n_assert=0; n_blind=0
for f in rust/scripts/*.sh scripts/*.sh; do
    det="$(grep -cE "$DET_RE" "$f" 2>/dev/null || true)"; det="${det:-0}"
    [[ "$det" -eq 0 ]] && continue
    n_assert=$((n_assert + 1))
    # ⛔ Only identity references INSIDE the comparison loop can pin the input a loop re-reads.
    # A pre-loop check (the SV cert gate's contract-identity call) verifies a DIFFERENT artifact
    # and is hoisted out of the loop, so it cannot see the input move between iterations.
    loop_start="$(grep -nE '^\s*(for|while) ' "$f" | grep -iE 'seed|lane|round' | head -1 | cut -d: -f1)"
    if [[ -n "$loop_start" ]]; then
        pin="$(tail -n +"$loop_start" "$f" | grep -cE "$PIN_RE" 2>/dev/null || true)"
    else
        pin="$(grep -cE "$PIN_RE" "$f" 2>/dev/null || true)"
    fi
    pin="${pin:-0}"
    if [[ "$pin" -eq 0 ]]; then verdict="⛔ BLIND"; n_blind=$((n_blind + 1)); else verdict="pins"; fi
    printf '%-42s %-14s %-14s %s\n' "$(basename "$f")" "$det ref(s)" "$pin ref(s)" "$verdict"
done
echo
echo "GATE-INPUT-PIN-CENSUS: asserts_determinism=${n_assert} blind_to_input_change=${n_blind}"
[[ "$n_blind" -eq 0 ]] && exit 0 || exit 1
