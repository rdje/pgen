// SV-CORPUS-GRAD.13c.2q — the same lost-digit collapse in `pass_en_switchtype`.
// IEEE 1364-2005 A.3.4: pass_en_switchtype ::= tranif0 | tranif1 | rtranif1 | rtranif0
module m;
  wire o, i, e;
  tranif0 g1(o, i, e);
endmodule
