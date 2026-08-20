module m;
  logic clk, a, b;
  property p; @(posedge clk) reject_on (b) a; endproperty
  assert property (p);
endmodule
