// SV-CORPUS-GRAD.3.35 - an INVALID pin - must keep REJECTING on every profile.
// [1:0] x; - a packed dimension with no data type and no `var`
// IEEE 1800-2017 Annex A footnote 6 docs/systemverilog/2017/md/section-41-data-read-api.md:3504-3505
module top;
  [1:0] x;
endmodule
