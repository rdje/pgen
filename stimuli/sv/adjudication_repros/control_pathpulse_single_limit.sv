// SV-CORPUS-GRAD.13c.2h row 2 — the LRM-AMBIGUOUS form.
// `(1)` is a legal constant_mintypmax_expression (A.8.4: constant_primary admits
// `( constant_mintypmax_expression )`), so BOTH alternatives of specparam_assignment derive this
// text and the standard supplies no rule to prefer one. PGEN resolves the exact tie to the EARLIER
// alternative, so it takes the ordinary route. Both readings are conformant; this row pins WHICH
// one PGEN chose, so the resolution is a ratchet rather than an accident.
module m;
  specparam PATHPULSE$clk$q = (1);
endmodule
