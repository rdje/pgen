// SV-CORPUS-GRAD.3.60 - NON-ANSI port declarations whose port NAME is a $unit typedef (iverilog
// sv_type_identifier_port_name.v:57 `input T;`, :73 `input T, U;`, :81 `input T x, T;`, :85 `input T T;`). IEEE 1800-2017
// A.2.1.2 input_declaration ::= input net_port_type list_of_port_identifiers | input variable_port_type
// list_of_variable_identifiers, net_port_type ::= [ net_type ] data_type_or_implicit - a data type is followed by a port
// identifier, so `input T;` is the implicit type + the port T; output_declaration / inout_declaration likewise.
typedef logic [3:0] T;
module m(T);
  input T;
endmodule
module m2(T, U);
  input T, U;
endmodule
module m3(x, T);
  input T x, T;
endmodule
module m4(T);
  input T T;
endmodule
module m5(T);
  output T;
endmodule
module m6(T);
  inout T;
endmodule
