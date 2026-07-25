#!/usr/bin/env bash
# SV-CORPUS-GRAD.3.11 — the IEEE 1800-2017 Annex-A `time_literal` repro matrix.
#
# ONE driver produces the BEFORE and the AFTER artifact, so the two are
# comparable by construction (the `.3.9`/`.3.10` discipline: a repro matrix that
# is RE-RUN, not re-typed).
#
# Usage:  bash run_matrix.sh <out.txt>
# Probe:  rust/target/release/parseability_probe (caller asserts it is NEWER
#         than generated/systemverilog_parser.rs).
set -uo pipefail

REPO=/Volumes/SSD/Documents/github/pgen
PROBE="$REPO/rust/target/release/parseability_probe"
OUT="${1:?usage: run_matrix.sh <out.txt>}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------------------
# wrappers — the THREE consumer contexts of `time_literal:495`
# ---------------------------------------------------------------------------
# tu  : timeunits_declaration:5569   `timeunit <lit>;`
tu_sample () { printf 'module m;\n  timeunit %s;\nendmodule\n' "$1"; }
# dly : delay_value_sv_only:1961     `#<lit>`
dly_sample () { printf 'module m;\n  initial #%s $display("x");\nendmodule\n' "$1"; }
# exp : primary_literal_sv_only:4264 expression position
exp_sample () { printf 'module m;\n  localparam p = %s;\nendmodule\n' "$1"; }

verdict () { # $1 = file, $2 = profile -> ACCEPT | REJECT
  if timeout 25 "$PROBE" --parse systemverilog "$1" --profile "$2" >/dev/null 2>&1; then
    echo ACCEPT
  else
    echo REJECT
  fi
}

row () { # $1 = id, $2 = wrapper, $3 = spelling, $4 = note
  local f="$WORK/$1.sv"
  case "$2" in
    tu)  tu_sample  "$3" > "$f" ;;
    dly) dly_sample "$3" > "$f" ;;
    exp) exp_sample "$3" > "$f" ;;
  esac
  printf '  %-5s %-5s %-14s %-7s %s\n' \
    "$1" "$2" "$(printf '%q' "$3")" "$(verdict "$f" sv_2017)" "$4"
}

src_row () { # $1 = id, $2 = note, stdin = the whole module source
  local f="$WORK/$1.sv"
  cat > "$f"
  printf '  %-5s %-7s %s\n' "$1" "$(verdict "$f" sv_2017)" "$2"
}

# the emitted `time_literal` shape, entry-relative (proves {value, unit} survives)
tl_shape () { # $1 = literal text
  local src="$WORK/shape.sv" dump="$WORK/shape.json"
  printf '%s' "$1" > "$src"
  "$PROBE" --parse-dump-ast systemverilog "$src" "$dump" --profile sv_2017 \
      --entry-rule time_literal >/dev/null 2>&1 || { echo "<reject>"; return; }
  python3 - "$dump" <<'PY'
import json, sys
def find(n):
    if isinstance(n, dict):
        if 'value' in n and 'unit' in n:
            return n
        for v in n.values():
            r = find(v)
            if r is not None:
                return r
    elif isinstance(n, list):
        for v in n:
            r = find(v)
            if r is not None:
                return r
    return None
node = find(json.load(open(sys.argv[1])))
print(json.dumps(node, separators=(',', ':')) if node else '<no {value,unit} node>')
PY
}

{
  echo "# SV-CORPUS-GRAD.3.11 — IEEE 1800-2017 Annex-A time_literal repro matrix"
  echo "# probe        : $PROBE"
  echo "# probe  mtime : $(date -r "$PROBE" '+%Y-%m-%d %H:%M:%S')"
  echo "# parser mtime : $(date -r "$REPO/generated/systemverilog_parser.rs" '+%Y-%m-%d %H:%M:%S')"
  echo "# grammar sha  : $(shasum -a 256 "$REPO/grammars/systemverilog.ebnf" | cut -c1-16)"
  echo
  echo "GOVERNING LAW — IEEE 1800-2017 Annex A (A.8.4 + footnote 44), verbatim:"
  echo '  time_literal44 ::= unsigned_number time_unit | fixed_point_number time_unit'
  echo '  time_unit      ::= s | ms | us | ns | ps | fs'
  echo '  44) "The unsigned number or fixed-point number in time_literal shall'
  echo '       not be followed by a white_space."'
  echo '  unsigned_number33    ::= decimal_digit { _ | decimal_digit }'
  echo '  fixed_point_number33 ::= unsigned_number . unsigned_number'
  echo '  33) "Embedded spaces are illegal."'
  echo
  echo '  => TWO independent constraints, both violated by `time_literal := number time_unit`:'
  echo '     (A) no white space at the number<->unit seam           [footnote 44]'
  echo '     (B) the number is unsigned_number OR fixed_point_number'
  echo '         -- NOT the full `number` (no based literals, no exponent reals) [A.8.4]'
  echo
  echo "================================================================================"
  echo "MUST ACCEPT — the LRM-legal spellings (regression guard for the fix)"
  echo "================================================================================"
  printf '  %-5s %-5s %-14s %-7s %s\n' id ctx spelling sv_2017 note
  row A1  tu   "10ns"        "unsigned_number + unit"
  row A2  tu   "1.5ns"       "fixed_point_number + unit"
  row A3  tu   "10_0ns"      "underscore separator (LRM unsigned_number)"
  row A4  tu   "1s"          "unit s"
  row A5  tu   "1ms"         "unit ms"
  row A6  tu   "1us"         "unit us"
  row A7  tu   "1ps"         "unit ps"
  row A8  tu   "1fs"         "unit fs"
  row A9  tu   "1_0.5_5ns"   "underscores both sides of the point"
  row A10 dly  "10ns"        "delay_value context"
  row A11 dly  "1.5ps"       "delay_value context, fixed point"
  row A12 exp  "10ns"        "expression context"
  row A13 exp  "1.5fs"       "expression context, fixed point"
  echo
  echo "================================================================================"
  echo "MUST REJECT (A) — footnote 44: white space at the number<->unit seam"
  echo "================================================================================"
  printf '  %-5s %-5s %-14s %-7s %s\n' id ctx spelling sv_2017 note
  row B1  tu   "10 ns"       "DEFECT — space"
  row B2  tu   "10	ns"       "DEFECT — tab"
  row B3  tu   "1.5 ns"      "DEFECT — fixed point + space"
  row B4  tu   "10  ns"      "DEFECT — multi-space"
  row B5  dly  "10 ns"       "DEFECT — delay context"
  row B6  exp  "10 ns"       "DEFECT — expression context"
  src_row B7 "DEFECT — newline at the seam (fn 44 says white_space, not just blanks)" <<'EOF'
module m;
  timeunit 10
  ns;
endmodule
EOF
  echo
  echo "================================================================================"
  echo "MUST REJECT (B) — A.8.4: the number is unsigned_number | fixed_point_number ONLY"
  echo "================================================================================"
  printf '  %-5s %-5s %-14s %-7s %s\n' id ctx spelling sv_2017 note
  row C1  tu   "1e3ns"       "DEFECT — exponent real_number is NOT fixed_point_number"
  row C2  tu   "1.5e3ns"     "DEFECT — exponent real_number"
  row C3  tu   "4'd10ns"     "DEFECT — based (decimal) integral_number"
  row C4  tu   "4'b10ns"     "DEFECT — based (binary) integral_number"
  row C5  tu   "'d10ns"      "DEFECT — unsized based integral_number"
  row C6  dly  "1e3ns"       "DEFECT — delay context"
  row C7  tu   "4'd10 ns"    "DEFECT — based AND spaced (both constraints)"
  echo
  echo "================================================================================"
  echo "MUST STAY REJECTED — over-tightening guard (already correct today)"
  echo "================================================================================"
  printf '  %-5s %-5s %-14s %-7s %s\n' id ctx spelling sv_2017 note
  row D1  tu   "10 n s"      "unit itself split — never legal"
  row D2  tu   "10zs"        "zs is not an LRM time_unit"
  row D3  tu   ".5ns"        "no leading digit — not unsigned_number"
  row D4  tu   "10"          "no unit at all"
  row D5  tu   "ns"          "no number at all"
  echo
  echo "================================================================================"
  echo "REAL-WORLD LEGAL CODE — the REJECTS-VALID face of the SAME defect"
  echo "================================================================================"
  echo "# Each shape below is verbatim-legal SV taken from the vendored corpus. It is"
  echo "# REJECTED today because time_literal STEALS <number> <ws> <unit-spelled-id>."
  echo "# Expected AFTER the fix: ACCEPT."
  src_row E1 "iverilog ivltests/pr2785294.v:22 shape — '#1 ps[idx] = ...' (ps = a reg array)" <<'EOF'
module m;
  reg [7:0] ps;
  integer idx;
  initial begin
    #1 ps[idx] = 1'b1;
  end
endmodule
EOF
  src_row E2 "Surelog tests/FSMBsp13/top.v:63 shape — '#2 s = ~s;' (s = a reg)" <<'EOF'
module m;
  reg s;
  initial forever begin
    #2 s = ~s;
  end
endmodule
EOF
  src_row E3 "ispras 16.08_04.sv:19 shape — 'a ##1 s ##1 b' (s = a sequence operand)" <<'EOF'
module m;
  logic clk, a, s, b;
  sequence r;
    @(posedge clk) a ##1 s ##1 b;
  endsequence
endmodule
EOF
  src_row E4 "the same SVA shape for EVERY time-unit spelling (ms/us/ns/ps/fs)" <<'EOF'
module m;
  logic clk, a, ms, us, ns, ps, fs, b;
  sequence r1; @(posedge clk) a ##1 ms ##1 b; endsequence
  sequence r2; @(posedge clk) a ##1 us ##1 b; endsequence
  sequence r3; @(posedge clk) a ##1 ns ##1 b; endsequence
  sequence r4; @(posedge clk) a ##1 ps ##1 b; endsequence
  sequence r5; @(posedge clk) a ##1 fs ##1 b; endsequence
endmodule
EOF
  src_row E5 "CONTROL — the same shapes with a NON-unit identifier (accept before AND after)" <<'EOF'
module m;
  reg [7:0] qs;
  reg t;
  integer idx;
  logic clk, a, zz, b;
  initial begin
    #1 qs[idx] = 1'b1;
    #2 t = ~t;
  end
  sequence r; @(posedge clk) a ##1 zz ##1 b; endsequence
endmodule
EOF
  echo
  echo "================================================================================"
  echo "TIME-LITERAL AST SHAPE — --entry-rule time_literal, the {value, unit} contract"
  echo "================================================================================"
  echo "# The fix must NOT flatten or renumber this object (SV-Slice-60 typed it"
  echo "# {value, unit}; delay_value_sv_only:1961 + primary_literal_sv_only:4264 wrap it)."
  for lit in "10ns" "1.5ns" "1ps" "10 ns"; do
    printf '  %-10s -> %s\n' "$(printf '%q' "$lit")" "$(tl_shape "$lit")"
  done
  echo
  echo "================================================================================"
  echo "NEIGHBOURING CONSTRUCT — 1step must not be collateral damage"
  echo "================================================================================"
  src_row F1 "delay_value 'step' alternative — 1step (NOT a time_literal)" <<'EOF'
module m;
  logic clk, d, q;
  always @(posedge clk) q <= #1step d;
endmodule
EOF
  echo
  echo "================================================================================"
  echo "THE FOUR REAL CORPUS FILES keyed to this defect"
  echo "================================================================================"
  for f in \
    stimuli/sv/subs/iverilog/ivtest/ivltests/pr2785294.v \
    stimuli/sv/subs/Surelog/tests/FSMBsp13/top.v \
    stimuli/sv/subs/ispras-sv-tests/ieee-1800-2012/16/16.08_04.sv \
    stimuli/sv/subs/ispras-sv-tests/ieee-1800-2012/16/16.09.11_01.sv ; do
    printf '  %-7s %s\n' "$(verdict "$REPO/$f" sv_2017)" "$f"
  done
} > "$OUT"

cat "$OUT"
