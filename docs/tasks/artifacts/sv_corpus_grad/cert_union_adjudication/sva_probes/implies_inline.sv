module m;
  logic clk, a, b;
  assert property (@(posedge clk) a implies b);
endmodule
