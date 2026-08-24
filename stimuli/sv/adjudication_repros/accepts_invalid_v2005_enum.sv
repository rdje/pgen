// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `enum` occurs 0 times in the whole Annex
// WHERE: same door as chandle: data_type alternative 5, AST kind:"enum"
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  enum { A } e;
endmodule
