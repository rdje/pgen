// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `iff` occurs 0 times; A.1.5:65 module_or_generate_item has no such item
// WHERE: checker_or_generate_item_declaration:1181 + module_or_generate_item_declaration:3778 + event_expression_primary; NOT declaration-shaped
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  default disable iff x;
endmodule
