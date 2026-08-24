// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `rand` occurs 0 times; 1364-2005 has no checker construct
// WHERE: checker_or_generate_item_declaration:1165's ( kw_rand )? optional. ⛔ `randc` shares random_qualifier but is NOT reachable under verilog_2005 - it arrives only via struct_union_member, whose alternative needs the sv-gated struct_union
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  generate rand integer x; endgenerate
endmodule
