module m;
  logic clk, a, b;
  property p; @(posedge clk) a implies b; endproperty
  assert property (p);
endmodule
