module m;
  logic clk, a, b, c;
  property p; @(posedge clk) a implies b until c; endproperty
  property q; @(posedge clk) a until b implies c; endproperty
  assert property (p);
  assert property (q);
endmodule
