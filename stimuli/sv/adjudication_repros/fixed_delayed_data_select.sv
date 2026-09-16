// SV-CORPUS-GRAD.3.63 — IEEE 1364-2005 A.7.5.2 / IEEE 1800-2017 A.7.5.2 `delayed_data ::= terminal_identifier |
// terminal_identifier [ constant_mintypmax_expression ]` — a SELECT on the delayed data terminal (literal
// brackets). Shape from iverilog/ivtest/ivltests/pr1520314.v (`$setuphold(..., D, BL_X[0]);`); through
// 1.0.230 the parse stopped at the `[`.
module m(input A, B, C, D, BL, BL_X);
  specify
    $setuphold(posedge A, BL, 0, 0, C,,,D, BL_X[0]);
  endspecify
endmodule
