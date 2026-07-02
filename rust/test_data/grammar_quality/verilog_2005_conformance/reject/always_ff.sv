module m;
  reg q;
  reg clk;
  always_ff @(posedge clk) q <= 1'b0;
endmodule
