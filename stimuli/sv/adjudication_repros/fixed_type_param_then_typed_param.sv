// SV-CORPUS-GRAD.3.35 - a FIXED pin - must keep ACCEPTING.
// module top #(int N = 5, M = N * 16, type T = int, T x = 0) (); - IEEE 1800-2017 section 6.20.1's own example: a typed parameter after a type parameter
// IEEE 1800-2017 section 6.20.1 docs/systemverilog/2017/md/section-6-data-types.md; A.1.3 parameter_port_list docs/systemverilog/2017/md/section-41-data-read-api.md:236-243, parameter_port_declaration ::= ... | data_type list_of_param_assignments | type list_of_type_assignments
module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();
endmodule
