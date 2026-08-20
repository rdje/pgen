module m;
  logic clk, a;
  property p; @(posedge clk) eventually [1:2] a; endproperty
  assert property (p);
endmodule
