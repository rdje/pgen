// SV-CORPUS-GRAD.3.35 - an INVALID pin - must keep REJECTING on every profile.
// y = 1; at module level - an implicit-type data declaration with an initializer and no `var`
// IEEE 1800-2017 Annex A footnote 6 docs/systemverilog/2017/md/section-41-data-read-api.md:3504-3505; A.1.4 module_common_item has no bare assignment item
module top;
  logic y;
  y = 1;
endmodule
