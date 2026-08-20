module m;
  logic clk, a, b;
  property p; @(posedge clk) sync_reject_on (b) a; endproperty
  assert property (p);
endmodule
