module m;
  logic a, b;
  property p;
    s_always [1:2] b;
  endproperty
  assert property (p);
endmodule
