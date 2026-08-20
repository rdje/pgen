module m;
  logic clk, a, b;
  property p; @(posedge clk) a iff b; endproperty
endmodule
