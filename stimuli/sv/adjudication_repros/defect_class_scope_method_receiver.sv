// SV-CORPUS-GRAD.13c.2j — CONFIRMED DEFECT (rejects-valid), routed by `.13c.2c`.
// IEEE 1800-2017 A.8.2: `method_call_root ::= primary | implicit_class_handle`, and A.8.4's
// `primary ::= … | [ class_qualifier | package_scope ] hierarchical_identifier select | …` with
// `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]`. So a CLASS-SCOPED
// name is a legal method-call receiver. PGEN's `primary_hier_scope_prefix` carries the
// `class_scope` branch (added by SV-EXH-PROOF.3.3.4.b.6.2.37.3 for a measured defect), but THREE
// hand-spelled copies of that prefix spell it INLINE with only two branches and never received it.
// ⭐ `.13c.2j` MEASURED which of the three decides this row, one site at a time through the
// interpreter: `split_hierarchical_callable_receiver` alone flips it; neither `method_call_receiver_*`
// copy moves it on either profile. The leaf opened naming the receiver copies — the right production,
// the wrong site.
// Before `.13c.2j`: REJECT (`position 125` on the comment-free minimal, 939 on this file).
// After: ACCEPT, via the `split_hierarchical` arm.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class base; static inner m; endclass
endpackage
module top;
  import p::*;
  int y;
  initial y = p::base::m.g();
endmodule
