// SV-CORPUS-GRAD.50 (release 1.0.253) — ILLEGAL under IEEE 1364-2005: TWO packed ranges on a named-block reg declaration.
// IEEE 1364-2005 A.2.8 block_item_declaration ::= { attribute_instance } reg [ signed ] [ range ] list_of_block_variable_identifiers ;
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172 reg_declaration carries the same ONE [ range ]; range :252); IEEE 1800-2017 A.2.2.1
// data_type ::= integer_vector_type [ signing ] { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:842) derives any number, which is why the same
// text is the sv_2017 / sv_2023 control control_sv_two_packed_dims_sites.sv. Through 1.0.252 block_data_type alternative 1
// served verilog_2005 with the 1800 repetition; packed_dimension_list_v2005 admits one range. iverilog 13.0 -g2005 accepts
// it (a tool extension; the clause decides).
module m;
  initial begin : b
    reg [1:0][1:0] q;
  end
endmodule
