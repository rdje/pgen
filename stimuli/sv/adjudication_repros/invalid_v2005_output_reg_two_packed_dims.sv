// SV-CORPUS-GRAD.50 (release 1.0.253) — ILLEGAL under IEEE 1364-2005: TWO packed ranges on an output reg port.
// IEEE 1364-2005 A.2.1.2 output_declaration ::= output [ net_type ] [ signed ] [ range ] list_of_port_identifiers | output
// reg [ signed ] [ range ] list_of_variable_port_identifiers | … (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:137-139) — ONE optional range; IEEE 1800-2017 A.2.2.1
// data_type ::= integer_vector_type [ signing ] { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:842) derives any number (control:
// control_sv_two_packed_dims_sites.sv). Through 1.0.252 data_type alternative 1 served verilog_2005 with the 1800 repetition
// (reached through output_variable_data_type_v2005); packed_dimension_list_v2005 admits one range. iverilog 13.0 -g2005 accepts it.
module m(output reg [1:0][1:0] q);
endmodule
