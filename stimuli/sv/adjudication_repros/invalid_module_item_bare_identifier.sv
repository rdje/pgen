// SV-CORPUS-GRAD.3.35 - an INVALID pin - must keep REJECTING on every profile.
// a; at module level - an implicit-type data declaration without `var` (Annex A footnote 6)
// IEEE 1800-2017 Annex A footnote 6 docs/systemverilog/2017/md/section-41-data-read-api.md:3504-3505: 'In a data_declaration, it shall be illegal to omit the explicit data_type before a list_of_variable_decl_assignments unless the var keyword is used'
module top;
  a;
endmodule
