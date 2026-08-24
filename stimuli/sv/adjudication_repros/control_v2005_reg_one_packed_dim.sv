// SV-CORPUS-GRAD.13e.7 — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.2.1.3 reg_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172) ::= reg [ signed ] [ range ] list_of_variable_identifiers ;
// WHY: pins the reg_two_packed_dims gate to the CARDINALITY and not to the dimension syntax
module test;
  reg [7:0] r;
  initial r = 0;
endmodule
