// SV-CORPUS-GRAD.50 (release 1.0.253) — ILLEGAL under IEEE 1364-2005: TWO packed ranges as a function's range.
// IEEE 1364-2005 A.2.6 function_range_or_type ::= [ signed ] [ range ] | integer | real | realtime | time (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:268-270;
// function_declaration :254-258) — ONE optional range; IEEE 1800-2017 A.2.6 function_data_type_or_implicit ::= data_type_or_void
// | implicit_data_type with implicit_data_type ::= [ signing ] { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:1084, :867) derives any number
// (the sv_2017 / sv_2023 control is control_sv_two_packed_dims_sites.sv). Through 1.0.252 implicit_data_type served
// verilog_2005 with the 1800 repetition; packed_dimension_list_v2005 admits one range. iverilog 13.0 -g2005 accepts it.
module m;
  function [1:0][1:0] f;
    input a;
    f = a;
  endfunction
endmodule
