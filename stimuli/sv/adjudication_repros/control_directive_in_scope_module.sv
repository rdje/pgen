module m;
  `celldefine
  wire a;
  `endcelldefine
  `timescale 1ns / 1ps
  `pragma protect end
  `line 5 "foo" 2
  `undef X
  wire b;
endmodule
