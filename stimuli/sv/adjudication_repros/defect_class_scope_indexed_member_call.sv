// SV-CORPUS-GRAD.13c.2j — the THIRD rendering of A.8.4's `[ class_qualifier | package_scope ]`.
// `.13c.2j` was opened against the two `method_call_receiver_sv_*` copies; the sweep for the class
// found a third hand-spelled copy at `split_hierarchical_callable_receiver` — the very rule
// `.13c.2c` had just fixed — carrying the identical 2-branch prefix with no `class_scope`.
// This reproducer adds an INDEXED member component to the class-scoped receiver, so it pins the
// COMPOSITION of the two fixes that rule has now received: `.13c.2c`'s member loop (which only runs
// because its dead guard was replaced) and `.13c.2j`'s `class_scope` prefix. Either one missing and
// this text has no derivation.
// Expected before `.13c.2j`: REJECT. Expected after: ACCEPT, via the `split_hierarchical` arm.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class holder; inner arr[2]; endclass
  class base; static holder h; endclass
endpackage
module top;
  import p::*;
  int y;
  initial y = p::base::h.arr[0].g();
endmodule
