// SV-CORPUS-GRAD.13c.2k — `instance` is a RESERVED KEYWORD in IEEE 1800-2023 Annex B (line 113)
// AND in IEEE 1364-2005 Annex B (line 59), so it cannot name an instance on any profile PGEN ships.
// Shape taken from iverilog/ivtest/ivltests/pr1758122.v, which PGEN accepted until .13c.2k.
module adder #(parameter W = 1) (input wire [W-1:0] A, output wire [W-1:0] Y);
  assign Y = A;
endmodule
module top;
  wire [2:0] a, y;
  adder #(3) instance(a, y);
endmodule
