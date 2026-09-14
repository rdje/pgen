// SV-CORPUS-GRAD.3.60 - an ANSI PORT named after a $unit typedef (iverilog sv_type_identifier_port_name.v:16).
// A.1.3 ansi_port_declaration ::= [ net_port_header ] port_identifier ..., net_port_header ::= [ port_direction ]
// net_port_type, net_port_type ::= [ net_type ] data_type_or_implicit - a data type is followed by the port identifier, so
// `input T` is direction + implicit type + port T;
// and the BARE continuation `input clk, T` is the same declaration with an all-empty header (both rejected through 1.0.229).
typedef logic [3:0] T;
module m(input T);
endmodule
module m2(input clk, T);
endmodule
