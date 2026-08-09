#!/usr/bin/env bash
# SV-CORPUS-GRAD.3.14b — the in-scope-directive control matrix.
#
# ⛔ GROUND TRUTH IS PINNED INSIDE THE INSTRUMENT ([[feedback_instrument_needs_ground_truth]]).
# The first version of this harness used `printf '%s'`, which writes a LITERAL `\n`; every
# case collapsed to one line, every case rejected, and the whole "expect=REJECT" half
# passed VACUOUSLY. The controls below would have caught it on the first run, so they now
# run FIRST and the harness REFUSES to report a matrix if either control misses.
set -uo pipefail
P=./rust/target/release/parseability_probe
W=tmp/ch22b/matrix
mkdir -p "$W"
fails=0

verdict() { # body, profile -> PASS|REJECT
  printf '%b' "$1" > "$W/t.sv"
  if timeout 60 "$P" --parse systemverilog "$W/t.sv" --profile "$2" >/dev/null 2>&1
  then echo PASS; else echo REJECT; fi
}

row() { # label, expect, body, profile
  local got; got=$(verdict "$3" "$4")
  local s="ok  "
  if [ "$got" != "$2" ]; then s="MISS"; fails=$((fails+1)); fi
  printf '%s  %-40s %-12s expect=%-6s got=%s\n' "$s" "$1" "$4" "$2" "$got"
}

echo "=== CONTROLS (the harness refuses to continue if either misses) ==="
c_pos=$(verdict 'module m;\nendmodule\n' sv_2017)          # must PASS
c_neg=$(verdict 'module m;\nendmodul\n'  sv_2017)          # must REJECT
printf 'positive control  "module m; endmodule"      -> %s (must be PASS)\n'   "$c_pos"
printf 'negative control  "module m; endmodul"       -> %s (must be REJECT)\n' "$c_neg"
if [ "$c_pos" != PASS ] || [ "$c_neg" != REJECT ]; then
  echo "⛔ CONTROL MISS — the instrument is not measuring what it claims. REFUSING."
  exit 3
fi

echo
echo "=== A. TOLERATED in a MODULE body — expect PASS ==="
row '`celldefine'          PASS 'module m;\n`celldefine\nendmodule\n'              sv_2017
row '`endcelldefine'       PASS 'module m;\n`endcelldefine\nendmodule\n'           sv_2017
row '`timescale'           PASS 'module m;\n`timescale 1ns / 1ps\nendmodule\n'     sv_2017
row '`pragma'              PASS 'module m;\n`pragma protect\nendmodule\n'          sv_2017
row '`line'                PASS 'module m;\n`line 1 "f.sv" 0\nendmodule\n'         sv_2017
row '`undef'               PASS 'module m;\n`undef FOO\nendmodule\n'               sv_2017
row '`undefineall (SV)'    PASS 'module m;\n`undefineall\nendmodule\n'             sv_2017

echo
echo "=== B. TOLERATED in a CLASS body — expect PASS ==="
row '`undef in class'      PASS 'class c;\n`undef FOO\nendclass\n'                 sv_2017
row '`pragma in class'     PASS 'class c;\n`pragma protect\nendclass\n'            sv_2017

echo
echo "=== C. PLACEMENT-RESTRICTED in a module body — expect REJECT (LRM forbids) ==="
row '`resetall  (22.3/19.6)'          REJECT 'module m;\n`resetall\nendmodule\n'                       sv_2017
row '`default_nettype (22.8/19.2)'    REJECT 'module m;\n`default_nettype none\nendmodule\n'           sv_2017
row '`unconnected_drive (22.9/19.9)'  REJECT 'module m;\n`unconnected_drive pull1\nendmodule\n'        sv_2017
row '`nounconnected_drive (22.9)'     REJECT 'module m;\n`nounconnected_drive\nendmodule\n'            sv_2017
row '`begin_keywords (22.14/19.11)'   REJECT 'module m;\n`begin_keywords "1800-2017"\nendmodule\n'     sv_2017
row '`end_keywords (22.14/19.11)'     REJECT 'module m;\n`end_keywords\nendmodule\n'                   sv_2017

echo
echo "=== D. TEXT-HIDING / TEXT-REWRITING in a module body — expect REJECT ==="
row '`include'  REJECT 'module m;\n`include "x.svh"\nendmodule\n'  sv_2017
row '`define'   REJECT 'module m;\n`define FOO 1\nendmodule\n'     sv_2017
row '`ifdef'    REJECT 'module m;\n`ifdef X\nendmodule\n'          sv_2017
row '`ifndef'   REJECT 'module m;\n`ifndef X\nendmodule\n'         sv_2017
row '`else'     REJECT 'module m;\n`else\nendmodule\n'             sv_2017
row '`elsif'    REJECT 'module m;\n`elsif Y\nendmodule\n'          sv_2017
row '`endif'    REJECT 'module m;\n`endif\nendmodule\n'            sv_2017

echo
echo "=== E. NOT DIRECTIVES — unexpanded MACRO uses stay REJECT (no over-acceptance) ==="
row 'user macro in module'  REJECT 'module m;\n`MY_MACRO(x)\nendmodule\n'                sv_2017
row 'uvm macro in class'    REJECT 'class c;\n`uvm_component_utils(c)\nendclass\n'       sv_2017
row '`__FILE__ (22.13)'     REJECT 'module m;\n`__FILE__\nendmodule\n'                   sv_2017
row '`__LINE__ (22.13)'     REJECT 'module m;\n`__LINE__\nendmodule\n'                   sv_2017
row 'name-prefix guard'     REJECT 'module m;\n`timescale_like_macro\nendmodule\n'       sv_2017
row 'name-prefix guard 2'   REJECT 'module m;\n`undef_not_a_directive\nendmodule\n'      sv_2017

echo
echo "=== F. TOP LEVEL is UNCHANGED (source_text_item already tolerated these) ==="
row '`timescale top-level'        PASS 'module m;\nendmodule\n`timescale 1ns / 1ps\n'    sv_2017
row '`default_nettype top-level'  PASS '`default_nettype none\nmodule m;\nendmodule\n'   sv_2017

echo
echo "=== G. PROFILE GATING ==="
row '`timescale in module (v2005)'   PASS   'module m;\n`timescale 1ns / 1ps\nendmodule\n' verilog_2005
row '`undefineall in module (v2005)' REJECT 'module m;\n`undefineall\nendmodule\n'         verilog_2005
row '`timescale in module (2023)'    PASS   'module m;\n`timescale 1ns / 1ps\nendmodule\n' sv_2023
row '`undefineall in module (2023)'  PASS   'module m;\n`undefineall\nendmodule\n'         sv_2023

echo
if [ "$fails" -eq 0 ]; then echo "MATRIX: all rows match expectation (0 misses)"; else echo "MATRIX: $fails MISS row(s)"; fi
exit $(( fails > 0 ))
