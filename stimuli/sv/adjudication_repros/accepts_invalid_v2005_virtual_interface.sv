// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `virtual` occurs 0 times in the whole Annex; 1364-2005 has no interface construct at all
// WHERE: same door: data_type alternative, AST kind:"virtual_interface"
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  virtual interface I vi;
endmodule
