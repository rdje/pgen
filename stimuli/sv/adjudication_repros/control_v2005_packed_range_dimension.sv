// SV-CORPUS-GRAD.3.38 - CONTROL (IEEE 1364-2005 profile): a sized packed range is unchanged under verilog_2005, where
// unsized_dimension is gated out entirely (.6.9) - the range arm's `!unsized_dimension` lookahead meets a rule that
// cannot match there and passes. Also accepted under sv_2017 / sv_2023. Arm !unsized.
// IEEE 1364-2005 A.2.5 range ::= [ msb_constant_expression : lsb_constant_expression ]
// docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:310
module m (input [3:0] p);
  reg [7:0] x;
  wire [3:0] y [0:1];
  reg signed [1:0] z;
endmodule
