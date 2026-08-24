// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.6.3 par_block (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:529) ::= fork [ : block_identifier { block_item_declaration } ] { statement } join - `join` is the ONLY terminator
// WHERE: join_keyword; a STATEMENT, so no declaration-shaped rewrite can reach it
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  initial fork join_any
endmodule
