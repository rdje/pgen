// SV-CORPUS-GRAD.13c.2j — CONFIRMED DEFECT (rejects-valid), routed by `.13c.2c`.
// IEEE 1800-2017 A.8.2: `method_call_root ::= primary | implicit_class_handle`, and A.8.4's
// `primary ::= … | [ class_qualifier | package_scope ] hierarchical_identifier select | …` with
// `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]`. So a CLASS-SCOPED
// name is a legal method-call receiver. PGEN's `primary_hier_scope_prefix` carries the
// `class_scope` branch (added by SV-EXH-PROOF.3.3.4.b.6.2.37.3 for a measured defect), but the
// hand-copied `method_call_receiver_sv_2017`/`_sv_2023` spell the prefix INLINE with only two
// branches and never received it.
// Expected today: REJECT (`Parser did not consume full input at position 125`).
// Expected once `.13c.2j` lands: ACCEPT.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class base; static inner m; endclass
endpackage
module top;
  import p::*;
  int y;
  initial y = p::base::m.g();
endmodule
