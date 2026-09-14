// SV-CORPUS-GRAD.3.60 - a PARAMETER PORT typed by a $unit typedef after a default-less-looking comma
// (iverilog sv_type_identifier_parameter_port_name.v:24 `T typed_value = 8'ha5`, :26 `p::PT pkg_value = 6'h2a`).
// A.1.3 parameter_port_declaration ::= parameter_declaration | data_type list_of_param_assignments | ...; A.2.4
// param_assignment ::= parameter_identifier { unpacked_dimension } [ = constant_param_expression ] (footnote 18: the
// default may be omitted only here) - a default-less assignment is COMPLETE only before `,` `)` or `;`, so after `, ` an
// identifier followed by an identifier or by `::` is the next `data_type list_of_param_assignments` declaration.
package p;
  typedef logic [5:0] PT;
endpackage
typedef logic [7:0] T;
module m1 #(parameter int P = 5, T typed_value = 8'ha5) ();
endmodule
module m2 #(parameter int P = 5, int R = 13, T typed_value = 8'ha5) ();
endmodule
module m3 #(parameter int P = 5, T T = 8'h3c, p::PT pkg_value = 6'h2a) ();
endmodule
