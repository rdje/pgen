module m;
  logic clk, a, b;
  property p; @(posedge clk) a s_until_with b; endproperty
endmodule
