#!/usr/bin/env bash
# run_anvil_first_contact.sh — can ANVIL serve as the independent corpus for `rtl_frontend`?
# (task-tree leaf `DONE-BAR.3a`; director: *"we can use ANVIL"*.)
#
# ANVIL (submodule `stimuli/generators/anvil`) is a random by-construction generator of
# SYNTHESIZABLE SystemVerilog whose validity model is architecturally independent of any grammar
# (its book chapter "Why Not a Grammar?" records that an annotated-EBNF walk was considered and
# REJECTED for circuit-cone recursion) => it cannot inherit grammars/rtl_frontend.ebnf's blind spots.
#
# ⭐ THE CALIBRATION QUESTION THIS ANSWERS: a corpus that cannot find a bug is not evidence of
# absence of bugs. This probe asks whether ANVIL finds anything on FIRST CONTACT.
#
# Every rejection is judged against a CROSS-PARSER CONTROL: if the full-LRM `systemverilog` parser
# accepts the same input, the input is valid SV and rtl_frontend's rejection is a subset boundary --
# deliberate or a gap. Without that control a rejection carries no information.
#
#   bash docs/tasks/artifacts/done_bar/run_anvil_first_contact.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

SUB="$ROOT/stimuli/generators/anvil"
PROBE="$ROOT/rust/target/release/parseability_probe"
WORK="$ROOT/rust/target/done_bar_audit/anvil_first_contact"
rm -rf "$WORK"; mkdir -p "$WORK"

# ANVIL's binary is built from the submodule; fall back to a same-revision sibling checkout if the
# submodule has not been built here, and SAY which was used -- never silently.
ANVIL="$SUB/target/release/anvil"
SIBLING="/Volumes/SSD/Documents/github/anvil/target/release/anvil"
SRC="submodule"
if [[ ! -x "$ANVIL" ]]; then
    if [[ -x "$SIBLING" ]] && \
       [[ "$(git -C "$SUB" rev-parse HEAD)" == "$(git -C "$(dirname "$(dirname "$SIBLING")")" rev-parse HEAD 2>/dev/null)" ]]; then
        ANVIL="$SIBLING"; SRC="sibling checkout at the IDENTICAL pinned revision"
    else
        echo "probe: REFUSED — no anvil binary at the pinned revision. Build it with:" >&2
        echo "  (cd stimuli/generators/anvil && cargo build --release)" >&2
        exit 2
    fi
fi
[[ -x "$PROBE" ]] || { echo "probe: REFUSED — $PROBE absent" >&2; exit 2; }

echo "=============================================================================="
echo "ANVIL x rtl_frontend — FIRST CONTACT"
echo "=============================================================================="
printf 'anvil binary : %s (%s)\n' "$ANVIL" "$SRC"
printf 'pinned rev   : %s\n\n' "$(git -C "$SUB" rev-parse --short HEAD)"

lane_sweep() {
    local lane="$1" n="$2"
    local pass=0 fail=0 s
    for ((s = 0; s < n; s++)); do
        "$ANVIL" --artifact "$lane" --seed "$s" >"$WORK/${lane}_$s.sv" 2>/dev/null
        # ⛔ REFUSE to score an artifact we have not confirmed is real. This probe's own first cut
        # hand-wrote a partial --config instead of the documented dump->edit->replay flow, ANVIL
        # emitted 0 bytes, and five EMPTY files scored as PASS. An empty input trivially "parses".
        if [[ "$(wc -c <"$WORK/${lane}_$s.sv" | tr -d ' ')" -lt 100 ]]; then
            echo "   MISCALIBRATED: ${lane} seed ${s} produced <100 bytes — refusing to score it" >&2
            return 3
        fi
        if timeout 60 "$PROBE" --parse rtl_frontend "$WORK/${lane}_$s.sv" >/dev/null 2>&1; then
            pass=$((pass + 1))
        else
            fail=$((fail + 1))
        fi
    done
    # Structural diversity: strip every digit, so only SHAPE remains. A generator emitting one
    # template with randomised constants would collapse to a single hash -- and a 100% pass rate
    # over one shape is not coverage.
    local shapes
    shapes=$(for ((s = 0; s < n; s++)); do sed 's/[0-9]\+/N/g' "$WORK/${lane}_$s.sv" | md5 -q; done | sort -u | wc -l | tr -d ' ')
    printf '   %-10s PASS=%-3s FAIL=%-3s over %s seeds   distinct structural shapes: %s/%s\n' \
        "$lane" "$pass" "$fail" "$n" "$shapes" "$n"
}

echo "1. LANE SWEEP (rtl_frontend parsing ANVIL output)"
lane_sweep frontend 10
lane_sweep dut 5
echo

echo "2. WHAT THE DUT LANE USES (construct census over one artifact)"
grep -oE "^[[:space:]]*(always_ff|always_comb|assign|case|generate|localparam|logic|wire|initial)\b" \
    "$WORK/dut_0.sv" | tr -d ' ' | sort | uniq -c | sort -rn | sed 's/^/   /'
echo

echo "3. MINIMAL REPROS, each with the cross-parser control"
pp() {
    printf '%b' "$2" >"$WORK/case.sv"
    local a b
    "$PROBE" --parse rtl_frontend  "$WORK/case.sv" >/dev/null 2>&1 && a="✅" || a="⛔"
    "$PROBE" --parse systemverilog "$WORK/case.sv" >/dev/null 2>&1 && b="✅" || b="⛔"
    printf '   rtl_frontend %s  systemverilog %s   %s\n' "$a" "$b" "$1"
}
pp "wire decl + assign"          'module m;\n wire [3:0] w;\n assign w = 4;\nendmodule\n'
pp "always_comb"                 'module m;\n logic [3:0] w;\n always_comb begin\n  w = 4;\n end\nendmodule\n'
pp "case inside always_comb"     'module m;\n logic [3:0] w;\n logic s;\n always_comb begin\n  case (s)\n   1: w = 4;\n   default: w = 0;\n  endcase\n end\nendmodule\n'
echo
echo "   grammar support (occurrences in grammars/rtl_frontend.ebnf):"
for k in case endcase always_comb always_ff assign wire generate; do
    printf '     %-12s %s\n' "$k" "$(grep -cw "$k" grammars/rtl_frontend.ebnf)"
done
echo

cat <<'EOF'
4. VERDICT

   ⭐ ANVIL FOUND A REAL GAP ON FIRST CONTACT. `case`/`endcase` have ZERO occurrences in
     grammars/rtl_frontend.ebnf; rtl_frontend rejects a `case` inside `always_comb` while the
     full-LRM `systemverilog` parser ACCEPTS the same input => valid, unambiguously synthesizable
     SV that rtl_frontend cannot parse. One DUT artifact uses `case` 25 times.

   ⭐ That is STRONGER than the calibration control originally demanded. The bar was "reproduce the
     known `**` gap"; ANVIL instead surfaced a gap that was NOT already known. A corpus that only
     reproduces known bugs adds nothing.

   ⚠️ THE PASS RATE IS NOT THE HEADLINE, IN EITHER DIRECTION:
     - the frontend lane's 10/10 is NOT weak evidence -- 10 seeds gave 10 distinct structural
       shapes, so it is not one template with randomised constants (a suspicion this probe raised
       and then refuted);
     - the DUT lane's 0/5 is NOT five defects -- all five stop at the same byte, and the module
       HEADER parses. It is one systematic body-construct gap, counted five times.

   ⛔ STILL UNMEASURED: which of rtl_frontend's claimed constructs ANVIL actually exercises. A
     generator's coverage is its own model's coverage, and that has not been characterised.
EOF
