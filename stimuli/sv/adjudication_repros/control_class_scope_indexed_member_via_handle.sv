// Control for `defect_class_scope_indexed_member_call.sv`: the same indexed-member call, made
// through a local handle assigned from the same class-scoped name. It parsed BEFORE `.13c.2j` and
// still parses, so the flip is attributable to the receiver's missing `class_scope` branch alone —
// not to the index, not to the call, not to the package import.
package p;
  class inner; function int g(); return 0; endfunction endclass
  class holder; inner arr[2]; endclass
  class base; static holder h; endclass
endpackage
module top;
  import p::*;
  holder hh;
  int y;
  initial begin hh = p::base::h; y = hh.arr[0].g(); end
endmodule
