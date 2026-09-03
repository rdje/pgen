// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// output [2:1] z = r[2 :+ 1]; in a non-ANSI body - an initializer on an implicit-type net port (sv2v port_init_early.sv: output a = 1;)
// IEEE 1800-2017 A.2.1.2 output_declaration section-41-data-read-api.md:762-765; list_of_port_identifiers section-41-data-read-api.md:975 carries no initializer, list_of_variable_port_identifiers section-41-data-read-api.md:994 does but needs variable_port_type ::= var_data_type section-41-data-read-api.md:907-908; IEEE 1364-2005 A.2.1.2 identical. t_lint_colonplus_bad.v l.13 and sv2v test/error/port_init_early.sv l.3 (.13x.c).
module t (z);
  reg [3:0] r = 4'b1010;
  output [2:1] z = r[2 :+ 1];
endmodule
