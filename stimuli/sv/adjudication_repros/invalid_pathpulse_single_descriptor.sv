// A.7.5 gives exactly two alternatives: no descriptors, or BOTH of them.
// One descriptor is neither, so `PATHPULSE$clk` is an ordinary specparam identifier and
// `(2, 9)` is not a constant_mintypmax_expression.
module m;
  specparam PATHPULSE$clk = (2, 9);
endmodule
