// ENGINE-UNIVERSAL-SERVICES.13 — probe for SV cycles 6 and 7 (the property_expr knot).
//
// Cycle (7 rules, reported by 7 of them under sv_2017 and 6 under sv_2023):
//   prop_and_sv_2017 -> prop_primary_sv_2017 -> property_expr -> property_expr_sv_2017 ->
//   prop_until_sv_2017 -> prop_iff_sv_2017 -> prop_or_sv_2017 -> prop_and_sv_2017
//
// The left-recursive alternative is `property_expr implies property_expr`
// (grammars/systemverilog.ebnf:4726). ⛔ A bare `a -> b` does NOT exercise it: `->` is also an
// ordinary binary expression operator (IEEE 1800-2017 A.8.6), so `sequence_expr` consumes it and
// the probe would pass while the alternative stays dead — the SV-CORPUS-GRAD.13c.2b trap.
// This probe forces the property-level alternative by making the LEFT operand a property that is
// not an expression: `(a |=> b)` is `lparen property_expr rparen`, so only
// `property_expr implies property_expr` can attach the `-> c`.
//
// Expected today: REJECT under sv_2017 AND sv_2023 (furthest_position at the `->`).
// Expected once .13 lands: ACCEPT under both.
module m;
  logic a, b, c;
  property p; (a |=> b) -> c; endproperty
endmodule
