// Control for `defect_class_scope_method_receiver.sv`: the SAME class-scoped name `p::base::m`,
// used as a VALUE rather than as a method-call receiver. It parses, so the defect is the RECEIVER
// POSITION — not the `pkg::class::static_member` name, and not the package import.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class base; static inner m; endclass
endpackage
module top;
  import p::*;
  inner y;
  initial y = p::base::m;
endmodule
