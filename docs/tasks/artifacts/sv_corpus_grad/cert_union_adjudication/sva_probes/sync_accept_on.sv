module m;
  logic clk, a, b;
  property p; @(posedge clk) sync_accept_on (b) a; endproperty
  assert property (p);
endmodule
