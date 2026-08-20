module m;
  logic clk, a, b;
  property p; @(posedge clk) a until b; endproperty
  assert property (p);
endmodule
