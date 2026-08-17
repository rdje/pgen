// IEEE 1800-2023 §30.7.1's OWN example, verbatim, in a specify block.
// The corpus carries it twice: ispras-sv-tests ieee-1800-2012/30/30.07.01_01.sv
// and ieee-1364-2005/test_14_06_01_1.v, both `fail -> pass` at slice 4.
module m (input clk, data, clr, pre, output q);
  specify
    (clk => q) = 12;
    (data => q) = 10;
    (clr, pre *> q) = 4;
    specparam
      PATHPULSE$clk$q = (2,9),
      PATHPULSE$clr$q = (0,4),
      PATHPULSE$ = 3;
  endspecify
endmodule
