// IEEE 1800-2023 A.7.5 pulse_control_specparam, alternative 2:
//   PATHPULSE$specify_input_terminal_descriptor$specify_output_terminal_descriptor = ( … )
// REJECTED before SV-CORPUS-GRAD.13c.2f slice 4 — the extraction flattened both
// nonterminal references into the token text.
module m;
  specparam PATHPULSE$clk$q = (2, 9);
endmodule
