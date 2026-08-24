// SV-CORPUS-GRAD.13e.4 — the BLOCK-scope half of the same over-acceptance, reached through a
// DIFFERENT grammar rule (`block_data_type`, not `data_type`), so a fix applied to one site only
// would leave this one live.
// IEEE 1364-2005 A.2.8 block_item_declaration
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:314) is
//   { attribute_instance } integer list_of_block_variable_identifiers ;
// — no signing there either; :313's `reg [ signed ] [ range ]` is the only block form that takes it.
// FIXED by SV-CORPUS-GRAD.13e.4: REJECT FOREVER on verilog_2005, ACCEPT on sv_2017/sv_2023.
module test;
  initial begin : b
    integer signed i;
    i = 0;
  end
endmodule
