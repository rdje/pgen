// SV-CORPUS-GRAD.13e.4 — the neighbouring construct where signing IS legal in IEEE 1364-2005, so
// the .13e.4 gate is pinned to exactly one difference: the ATOM branch, not the VECTOR branch.
// A.2.1.3 reg_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172)
//   reg_declaration ::= reg [ signed ] [ range ] list_of_variable_identifiers ;
// derives `reg signed [7:0] r;`, and PGEN reaches it through `data_type`'s
// `integer_vector_type ( signing )? packed_dimension*` alternative — the sibling of the
// `integer_atom_type ( signing )?` alternative .13e.4 gates. The `arm` claim is what makes that
// precise: an ACCEPT alone would not say WHICH alternative parsed it.
// Expected FOREVER: ACCEPT on verilog_2005, through integer_vector>signed.
module test;
  reg signed [7:0] r;
  initial r = 0;
endmodule
