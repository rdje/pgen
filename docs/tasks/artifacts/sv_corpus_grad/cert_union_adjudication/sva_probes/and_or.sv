module m;
  logic clk, a, b;
  property p; @(posedge clk) a and b; endproperty
  property q; @(posedge clk) a or b; endproperty
  assert property (p);
  assert property (q);
endmodule
