// SV-CORPUS-GRAD.13e.2 — REJECT FOREVER under verilog_2005.
// `'x` is SystemVerilog's unbased_unsized_literal and has NO derivation in IEEE 1364-2005:
// A.8.7 decimal_number (…/section-Annex_A-normative-formal-syntax-definition.txt:978) admits an
// `x_digit` only AFTER a `decimal_base`, and binary_/octal_/hex_number (:983-985) each require
// their own base. The vendored compiler testifies against itself — stimuli/sv/subs/iverilog/
// lexor.lex:516 warns "Using SystemVerilog 'N bit vector. Use at least -g2005-sv" and temporarily
// sets `generation_flag = GN_VER2005_SV` (:522) just to lex it.
// ⛔ This is the ACTUAL construct behind the ivtest rows partsel_outside_const.v and
// partsel_outside_expr.v — NOT the part-select their names advertise. See the sibling control
// control_v2005_partsel_negative_base.sv, which parses.
// Expected FOREVER: REJECT on verilog_2005.
module test;
  reg [1:0] r;
  initial if (r !== 'x) r = 0;
endmodule
