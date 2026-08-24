// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `static` occurs 0 times; A.2.1.3:152 gives a declaration no lifetime
// WHERE: data_declaration_sv_2017's ( lifetime )? optional. ⛔ `lifetime` itself must STAY reachable - A.2.6:255 function [ automatic ] and A.2.7:276 task [ automatic ] are LEGAL v2005; the gate goes on the declaration-site OPTIONAL
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  static integer i;
endmodule
