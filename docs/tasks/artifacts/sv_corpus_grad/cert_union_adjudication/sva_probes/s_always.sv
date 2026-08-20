module m;
  logic clk, a;
  property p; @(posedge clk) s_always [1:2] a; endproperty
  assert property (p);
endmodule
