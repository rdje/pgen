// The CORPUS shape, verbatim from verilator/test_regress/t/t_specparam.v:14-15 —
// mintypmax triples in BOTH limit values, which is the only place the optional
// `, error_limit_value` arm is exercised by real third-party text.
module m;
  specparam PATHPULSE$a$b = (3.0:3.1:3.2, 4.0:4.1:4.2);
  specparam PATHPULSE$a$c = (3.0:3.1:3.2);
endmodule
