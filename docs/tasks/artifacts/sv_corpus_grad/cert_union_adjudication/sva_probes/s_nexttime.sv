module m;
  logic clk, a;
  property p; @(posedge clk) s_nexttime a; endproperty
  assert property (p);
endmodule
