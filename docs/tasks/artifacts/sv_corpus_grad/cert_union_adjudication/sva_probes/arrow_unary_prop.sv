module m;
  logic clk, b;
  property p; @(posedge clk) -> b; endproperty
endmodule
