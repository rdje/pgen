// SV-CORPUS-GRAD.50 (release 1.0.253) — ILLEGAL under IEEE 1364-2005: TWO packed ranges on an ANSI input port.
// IEEE 1364-2005 A.2.1.2 input_declaration ::= input [ net_type ] [ signed ] [ range ] list_of_port_identifiers (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:135;
// inout :133, output :137-139; range :252) — ONE optional range; IEEE 1800-2017 A.2.2.1 implicit_data_type ::= [ signing ]
// { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:867) derives any number (the sv_2017 / sv_2023 control is control_sv_two_packed_dims_sites.sv).
// Through 1.0.252 implicit_data_type served verilog_2005 with the 1800 repetition; packed_dimension_list_v2005 admits one.
// iverilog 13.0 -g2005 accepts it (a tool extension; the clause decides). Corpus: sv2v test/core/tf_unpacked_input.v is
// the same shape on a task port.
module m(input [7:0][3:0] a);
endmodule
