#!/usr/bin/env bash
# run_rtl_frontend_real_world_probe.sh — has `rtl_frontend` ever parsed a real design file?
# (task-tree leaf `DONE-BAR.3`; asked by the director 2026-07-29: *"is rtl_frontend sufficiently
# tested? Is it ok to release it as is?"*)
#
# WHY THIS EXISTS. `rtl_frontend` claims `Done`. Its entire proof surface is 130 hand-written samples
# in a curated manifest this project authored — **35 KB in total, mean 270 bytes, largest 687 bytes**.
# `DONE-BAR.1` measured that no external corpus is wired to it and no family-status gate computes its
# status. What NOBODY had ever done is the obvious thing: point it at a real design file. This does.
#
# ⛔ WHAT THIS IS NOT. It is a PROBE, not a conformance gate, and its result is not a defect count:
#   - `rtl_frontend` parses a SYNTHESIZABLE SUBSET that PGEN itself delimits. Rejecting a file that
#     uses non-synthesizable constructs is CORRECT behaviour, not a bug. `initial` for instance has
#     **0 occurrences** in `grammars/rtl_frontend.ebnf` — deliberately out of subset.
#   - It therefore reports a REJECTION RATE, and separately isolates minimal repros with a
#     cross-parser control, so a genuine gap can be told apart from a correct subset boundary.
#
#   bash docs/tasks/artifacts/done_bar/run_rtl_frontend_real_world_probe.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

PROBE="$ROOT/rust/target/release/parseability_probe"
WORK="$ROOT/rust/target/done_bar_audit/rtl_frontend_real_world"
rm -rf "$WORK"; mkdir -p "$WORK"

if [[ ! -x "$PROBE" ]]; then
    echo "audit: REFUSED — $PROBE is absent. Build it with:" >&2
    echo "  (cd rust && cargo build --release --features generated_parsers)" >&2
    exit 2
fi

echo "=============================================================================="
echo "rtl_frontend vs REAL vendored RTL — the measurement nothing in the flow makes"
echo "=============================================================================="
echo

# --- 0. Price the curated proof surface it DOES have -------------------------------------------
M="$ROOT/rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json"
echo '0. THE CURATED PROOF SURFACE (what `Done` currently rests on)'
jq -r '[.samples[].sample] |
  "   samples: \(length)   total: \(add|length) bytes   mean: \((add|length)/length|floor) bytes   largest: \(max_by(length)|length) bytes"' "$M"
printf '   accepts/rejects: %s / %s\n' \
    "$(jq -r '[.samples[]|select(.expected_parse_ok==true)]|length' "$M")" \
    "$(jq -r '[.samples[]|select(.expected_parse_ok==false)]|length' "$M")"
echo "   ⇒ every one of these was written by this project. None is a real design file."
echo

# --- 1. Real design files, as-is ----------------------------------------------------------------
echo "1. REAL DESIGN FILES, AS SHIPPED (vendored under stimuli/sv/subs — already in this repo)"
echo
mapfile -t FILES < <(find stimuli/sv/subs/Cores-VeeR-EL2/design stimuli/sv/subs/friscv/rtl \
    \( -name '*.sv' -o -name '*.v' \) 2>/dev/null | sort | head -20)
if [[ "${#FILES[@]}" -eq 0 ]]; then
    echo "   REFUSED: no vendored RTL found — the submodules are not checked out." >&2
    exit 2
fi
pass=0; fail=0
for f in "${FILES[@]}"; do
    if timeout 30 "$PROBE" --parse rtl_frontend "$f" >/dev/null 2>&1; then
        pass=$((pass + 1)); v="✅"; echo "${f}" >>"$WORK/accepted.txt"
    else
        fail=$((fail + 1)); v="⛔"
    fi
    printf '   %s %-52s %8s bytes\n' "$v" "${f#stimuli/sv/subs/}" "$(wc -c <"$f" | tr -d ' ')"
done
echo
printf '   PASS=%d  FAIL=%d  of %d real design files\n' "$pass" "$fail" "${#FILES[@]}"
if [[ -s "$WORK/accepted.txt" ]]; then
    echo "   ACCEPTED (named, because a rejection rate without the passing set is half a measurement):"
    sed 's|^|     |' "$WORK/accepted.txt"
fi
echo

# --- 2. Minimal repros with a CROSS-PARSER CONTROL ----------------------------------------------
# ⭐ The control is what makes each row interpretable: if `systemverilog` (the full-LRM parser)
# ACCEPTS the same input, the input is valid SV and rtl_frontend's rejection is a SUBSET BOUNDARY --
# either a deliberate one or a gap. Without the control, a rejection could just mean invalid input,
# and this probe's own first cut DID produce one such false finding.
echo '2. MINIMAL REPROS (cross-parser control: does the full-LRM `systemverilog` parser accept it?)'
echo
probe_pair() {
    local label="$1" src="$2" note="$3"
    printf '%b' "$src" >"$WORK/case.sv"
    local a b
    "$PROBE" --parse rtl_frontend  "$WORK/case.sv" >/dev/null 2>&1 && a="✅" || a="⛔"
    "$PROBE" --parse systemverilog "$WORK/case.sv" >/dev/null 2>&1 && b="✅" || b="⛔"
    printf '   rtl_frontend %s   systemverilog %s   %-34s %s\n' "$a" "$b" "$label" "$note"
}
probe_pair "params + ANSI ports"       'module m #(\n parameter W = 8\n)(\n input wire clk\n);\nendmodule\n' "baseline"
probe_pair "port width [W-1:0]"        'module m #(\n parameter W = 8\n)(\n input wire [W-1:0] d\n);\nendmodule\n' ""
probe_pair "port width [W/8-1:0]"      'module m #(\n parameter W = 8\n)(\n input wire [W/8-1:0] d\n);\nendmodule\n' ""
probe_pair "unpacked memory [255:0]"   'module m;\n logic [7:0] ram [255:0];\nendmodule\n' ""
probe_pair "power operator 2**8"       'module m;\n logic [7:0] ram [2**8-1:0];\nendmodule\n' "GAP: ** absent from the grammar"
probe_pair "initial block"             'module m;\n initial begin\n end\nendmodule\n' "by design: 'initial' is out of subset"
echo
printf "   occurrences of '**' in grammars/rtl_frontend.ebnf: %s\n" "$(grep -c '\*\*' grammars/rtl_frontend.ebnf)"
printf "   occurrences of 'initial' in grammars/rtl_frontend.ebnf: %s\n" "$(grep -cw 'initial' grammars/rtl_frontend.ebnf)"
echo

# --- 3. Verdict ---------------------------------------------------------------------------------
cat <<EOF
3. WHAT THIS DOES AND DOES NOT ESTABLISH

   ESTABLISHED:
   - ${fail} of ${#FILES[@]} real vendored design files are rejected.
   - The '**' power operator is absent from the grammar (0 occurrences) and rejected, while the
     full-LRM parser accepts the same input => a genuine subset gap, not an invalid input.
   - 'initial' is absent by design (0 occurrences) => rejecting it is CORRECT for a synthesizable
     subset, and part of the rejection rate above is therefore legitimate.

   NOT ESTABLISHED:
   - The SPLIT between deliberate subset boundaries and genuine gaps. Per-file root causes are not
     attributed here, and must not be inferred from the count.
   ⇒ That split is exactly what DONE-BAR.3 owes, and it cannot be guessed from a rejection rate.

   THE POINT: this measurement had never been made. rtl_frontend holds \`Done\` on 35 KB of examples
   this project wrote itself, and the first time it met a real design file was this probe.
EOF
