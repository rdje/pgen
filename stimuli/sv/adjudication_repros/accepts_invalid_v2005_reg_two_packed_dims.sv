// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed — FIXED by SV-CORPUS-GRAD.50
// (release 1.0.253): packed_dimension_list_v2005 admits ONE range at data_type alternative 1; REJECT from 1.0.253.
// 1364-2005 A.2.1.3 reg_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172) ::= reg [ signed ] [ range ] list_of_variable_identifiers ; - exactly ONE optional range
// WHERE: data_type alternative 1's `packed_dimension*`. The L3 class: the RIGHT keywords in the WRONG arity, so no keyword-level census can see it
// Pinned BEFORE the fix so the ratchet went RED the day it landed; flipped to class=invalid by .50.
module test;
  reg [7:0][3:0] r;
endmodule
