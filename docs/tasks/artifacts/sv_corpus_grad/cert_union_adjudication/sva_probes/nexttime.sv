module m;
  logic clk, a;
  property p; @(posedge clk) nexttime a; endproperty
  assert property (p);
endmodule
