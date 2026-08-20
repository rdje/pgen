module m;
  logic clk, a, b;
  property q; @(posedge clk) a; endproperty
  property p; @(posedge clk) q and b; endproperty
  assert property (p);
endmodule
