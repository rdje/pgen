// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `var` occurs 0 times; A.2.1.3:152 carries no qualifier
// WHERE: 7 v2005-live sites, the widest of the population: data_declaration_sv_2017, block_data_declaration_sv_2017, for_variable_declaration, tf_port_declaration, tf_port_item_v2005, tf_port_item_v2005_continuation, var_data_type
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  var integer i;
endmodule
