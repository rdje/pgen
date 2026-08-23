// SV-CORPUS-GRAD.13e.2 — the REFUTATION control: the construct the ivtest rows
// partsel_outside_const.v / partsel_outside_expr.v are NAMED for parses perfectly.
// An indexed part-select whose base is a negative unsized decimal constant, `arr[-'d1 +: 2]`, is
// A.8.1 `expression [ expression +: constant_expression ]` with A.8.3 `unary_operator primary`
// supplying the sign — nothing about it is outside IEEE 1364-2005.
// This row exists so the pin on those two files can never be misread as a part-select ruling:
// their rejection is the unbased unsized `'x` in their self-checking `if`, measured, and this
// control is what separates the two hypotheses.
module test;
  reg [1:0] arr = 1;
  wire [1:0] o = arr[-'d1 +: 2];
endmodule
