module m;
  logic a, b, clk;
  property p;
    @(posedge clk) (a dist {1 := 1, 0 := 3}) |-> b;
  endproperty
endmodule
