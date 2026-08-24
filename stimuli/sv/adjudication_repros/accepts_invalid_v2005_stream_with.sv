// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `with` occurs 0 times, and 1364-2005 has no streaming operator at all
// WHERE: stream_expression:5828, the second v2005-live `with` site; pinned separately because a one-site gate would leave this one open
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  initial x = { << { a with [1:0] } };
endmodule
