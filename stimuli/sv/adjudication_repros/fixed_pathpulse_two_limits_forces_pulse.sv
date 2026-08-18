// SV-CORPUS-GRAD.13c.2h row 1 — the UNAMBIGUOUS form.
// `(2,9)` is not an expression (no comma production), so only pulse_control_specparam derives it.
// This is the row that proves the .13c.2f slice-4 fix reaches the production, not merely the text.
module m;
  specparam PATHPULSE$clk$q = (2,9);
endmodule
