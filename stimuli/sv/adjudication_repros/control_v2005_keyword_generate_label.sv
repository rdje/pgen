// SV-CORPUS-GRAD.13c.2k — the verilog_2005 half of the pair; legal Verilog-2005.
module top;
  genvar i;
  generate
    for (i = 0; i < 4; i = i + 1) begin:byte
      wire [7:0] w;
    end
  endgenerate
endmodule
