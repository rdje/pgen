// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.6.3 par_block (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:529) - `join` is the ONLY terminator
// WHERE: join_keyword, the sibling branch of fork_join_any; pinned separately so a one-branch gate cannot look complete
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  initial fork join_none
endmodule
