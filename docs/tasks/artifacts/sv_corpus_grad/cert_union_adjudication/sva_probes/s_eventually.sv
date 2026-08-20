module m;
  logic clk, a;
  property p; @(posedge clk) s_eventually a; endproperty
  assert property (p);
endmodule
