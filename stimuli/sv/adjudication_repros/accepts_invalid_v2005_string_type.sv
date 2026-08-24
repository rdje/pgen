// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.8.8 (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:1218) string ::= " { Any_ASCII_Characters_except_new_line } " - `string` is the LITERAL, never a data type
// WHERE: data_type + block_data_type + casting_type; the keyword collides with a 1364-2005 NONTERMINAL name, which is why a name-only oracle would have missed it
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  string s;
endmodule
