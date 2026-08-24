// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `with` occurs 0 times; A.8.4:935 primary has no array-method form
// WHERE: array_manipulation_call:737. ⭐ FOUND BY THE CENSUS'S OWN MISSING-WITNESS REFUSAL, on its first real run - absent from every hand list that preceded it
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  initial x = a.sum with (item);
endmodule
