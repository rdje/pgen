// SV-CORPUS-GRAD.50 (release 1.0.253) — ILLEGAL under IEEE 1364-2005: TWO packed ranges on a task port.
// IEEE 1364-2005 A.2.7 tf_input_declaration ::= input [ reg ] [ signed ] [ range ] list_of_port_identifiers | input task_port_type
// list_of_port_identifiers (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:296-305; task_port_item :290-293) — ONE optional range; IEEE 1800-2017 A.2.7 tf_port_item ::=
// … data_type_or_implicit … (docs/systemverilog/2017/md/section-41-data-read-api.md:1152) with implicit_data_type ::= [ signing ] { packed_dimension } (:867) derives any number
// (control: control_sv_two_packed_dims_sites.sv). The body carries the one statement_or_null 1364-2005 requires (the null
// statement), so the ONLY refusal is the second range. Through 1.0.252 implicit_data_type served verilog_2005 with the 1800
// repetition; packed_dimension_list_v2005 admits one. iverilog 13.0 -g2005 accepts it. Corpus: sv2v test/core/tf_unpacked_input.v.
module m;
  task t(input [1:0][1:0] a);
    ;
  endtask
endmodule
