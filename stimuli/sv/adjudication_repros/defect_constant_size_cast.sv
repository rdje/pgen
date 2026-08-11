// SV-CORPUS-GRAD.13c.2 — CONFIRMED DEFECT (rejects-valid).
// A size cast is rejected inside a parameter's CONSTANT expression.
// IEEE 1800-2017 A.8.4: constant_primary ::= … | constant_cast | …
//                       constant_cast    ::= casting_type ' ( constant_expression )
// Expected today: REJECT.  Expected once fixed: ACCEPT.
package p;
  parameter logic [7:0] K = 8'(1);
endpackage
