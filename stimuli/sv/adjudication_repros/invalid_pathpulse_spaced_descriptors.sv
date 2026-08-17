// ⭐ THE over-acceptance guard for SV-CORPUS-GRAD.13c.2f slice 4.
// A.9.3 makes `PATHPULSE$clk$q` ONE simple identifier, so the spaced spelling is three
// tokens and is not a pulse_control_specparam. Composing the token out of grammar elements
// (`PATHPULSE$` descriptor `$` descriptor) would accept this, because `trivia` skips layout
// before every element — which is why the fix is ONE contiguous regex.
module m;
  specparam PATHPULSE$ clk $ q = (2, 9);
endmodule
