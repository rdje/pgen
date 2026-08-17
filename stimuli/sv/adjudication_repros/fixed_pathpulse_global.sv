// IEEE 1800-2023 A.7.5 pulse_control_specparam, alternative 1:
//   PATHPULSE$ = ( reject_limit_value [ , error_limit_value ] )
// REJECTED before SV-CORPUS-GRAD.13c.2f slice 4 — the terminal matched the literal
// characters `PATHPULSE_dollar`, the extraction's transliteration of the LRM's `$`.
module m;
  specparam PATHPULSE$ = (1, 2);
endmodule
