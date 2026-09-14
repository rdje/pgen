// SV-CORPUS-GRAD.3.60 - a NET named after a $unit typedef (iverilog ivtest/ivltests/sv_type_identifier_net_name.v:15).
// IEEE 1800-2017 3.13: the compilation-unit scope name space and the module name space are distinct, so `T` the
// net is a new name; A.2.1.3 net_declaration ::= net_type ... data_type_or_implicit ... list_of_net_decl_assignments
// with data_type_or_implicit ::= data_type | implicit_data_type (A.2.2.1) - a data type in a declaration is followed by
// its declarator, so a typedef name followed by `=` is the DECLARATOR under the implicit type: `wire T = 1'b1;`.
typedef int T;
module m;
  wire T = 1'b1;
endmodule
