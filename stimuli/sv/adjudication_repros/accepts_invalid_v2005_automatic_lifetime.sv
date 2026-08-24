// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.2.1.3 integer_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:152) carries no lifetime; `automatic` is legal ONLY on function (A.2.6:255) and task (A.2.7:276)
// WHERE: the L2 class: `automatic` IS a 1364-2005 Annex B keyword, admitted in a POSITION the LRM never allows - invisible to a keyword-level census, which is why the census names L2 as an honest bound
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  automatic integer i;
endmodule
