// SV-CORPUS-GRAD.3.63 — A.7.5.2 `delayed_reference ::= terminal_identifier | terminal_identifier [ constant_mintypmax_expression ]`
// — the same literal-bracket select on the delayed REFERENCE terminal; through 1.0.230 the parse stopped at the `[`.
module m(input A, B, C, D, BL, BL_X);
  specify
    $setuphold(posedge A, BL, 0, 0, C,,,D[0], BL_X);
  endspecify
endmodule
