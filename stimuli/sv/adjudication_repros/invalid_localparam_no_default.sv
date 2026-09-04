// SV-CORPUS-GRAD.3.35 - an INVALID pin - must keep REJECTING on every profile.
// localparam X; - a localparam without its default (Annex A footnote 18)
// IEEE 1800-2017 Annex A footnote 18 docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520: the default may be omitted only within a parameter_port_list, and never from a localparam
module top;
  localparam X;
endmodule
