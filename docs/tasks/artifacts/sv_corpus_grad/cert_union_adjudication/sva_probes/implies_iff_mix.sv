module m;
  logic clk, a, b, c;
  property p; @(posedge clk) a iff b implies c; endproperty
  assert property (p);
endmodule
