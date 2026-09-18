import q::*;
module m;
  property p(a, b);
    a |-> b;
  endproperty
  sequence s(x);
    x ##1 x;
  endsequence
endmodule
