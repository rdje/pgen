// SV-CORPUS-GRAD.13c.2f slice 4 — the NARROWING half of PGEN-SV-CORPUS-GRAD-0221 (release 1.0.185,
// ledger SV-0055). Before that fix the grammar matched `PATHPULSE$` as the literal word
// `PATHPULSE_dollar` — a transliteration, not a transcription — so `PATHPULSE_dollar` was a MAGIC
// TOKEN and this text parsed as a `pulse_control_specparam` on all three profiles.
//
// It is not legal SystemVerilog. `PATHPULSE_dollar` is an ordinary simple identifier, so A.7.5
// gives `specparam_assignment ::= specparam_identifier = constant_mintypmax_expression`, and
// `(1, 2)` is not a constant_mintypmax_expression. The `pulse_control_specparam` alternative needs
// the literal `PATHPULSE$`, which this file does not contain.
//
// ⛔ PINNED BY .13c.2l: the fix shipped with five ACCEPT witnesses and NONE for the direction that
// can break a consumer. Expected REJECT FOREVER on all three profiles.
module m;
  specify
    specparam PATHPULSE_dollar = (1, 2);
  endspecify
endmodule
