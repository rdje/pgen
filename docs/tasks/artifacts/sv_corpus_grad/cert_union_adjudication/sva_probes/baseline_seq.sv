module m;
  logic clk, a, b;
  property p; @(posedge clk) a |-> b; endproperty
  assert property (p);
endmodule
