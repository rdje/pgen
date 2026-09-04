// SV-CORPUS-GRAD.3.35 - an INVALID pin - must keep REJECTING on every profile.
// localparam type X; - a localparam type without its default
// IEEE 1800-2017 Annex A footnote 18 docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520
module top;
  localparam type X;
endmodule
