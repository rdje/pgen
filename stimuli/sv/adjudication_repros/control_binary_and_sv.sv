// SV-CORPUS-GRAD.13e.13 — the control for invalid_binary_reduction_nand.sv on the IEEE 1800
// profiles: the same statement with a LISTED binary operator (A.8.6 binary_operator,
// docs/systemverilog/2017/md/section-11-operators-and-expressions.md:136-140). ACCEPT on sv_2017 / sv_2023.
module t;
  reg [7:0] a, b, c;
  initial c = a & b;
endmodule
