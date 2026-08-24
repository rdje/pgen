// SV-CORPUS-GRAD.13e.7 — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.2.1.3 (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:152 integer, :172 reg, and the real/realtime/time/event siblings beside them)
// WHY: ⛔ THE LOAD-BEARING CONTROL for .13e.7's whole fix: these are exactly why data_declaration_sv_2017 was admitted to verilog_2005 in the first place. If any per-site gate is drawn too wide, this row goes RED
module test;
  integer i;
  time t;
  real x;
  realtime rt;
  event e;
  reg r;
endmodule
