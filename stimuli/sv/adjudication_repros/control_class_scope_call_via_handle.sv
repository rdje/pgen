// Second control for `defect_class_scope_method_receiver.sv`: the same call `.g()`, made through a
// local handle assigned from the same class-scoped name. Both halves parse in isolation; only their
// composition — the class-scoped name IN the receiver position — rejects.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class base; static inner m; endclass
endpackage
module top;
  import p::*;
  inner h;
  int y;
  initial begin h = p::base::m; y = h.g(); end
endmodule
