module m;
  logic a, b;
  property p;
    eventually [1:2] b;
  endproperty
  assert property (p);
endmodule
