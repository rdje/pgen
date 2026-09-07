class C;
  `celldefine
  int a;
  `endcelldefine
  `undefineall
  `timescale 1ns / 1ps
  `pragma protect end
  `line 5 "foo" 2
  `undef X
  int b;
endclass
