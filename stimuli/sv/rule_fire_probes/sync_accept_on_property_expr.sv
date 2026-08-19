module m;
  logic a, b;
  property p;
    sync_accept_on (a) b;
  endproperty
  assert property (p);
endmodule
