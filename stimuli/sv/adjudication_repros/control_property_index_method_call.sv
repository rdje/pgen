// Control for `defect_tfport_index_method_call.sv`: the SAME expression parses when `ral` is a
// class PROPERTY rather than a subroutine formal, so the defect is the receiver's declaration
// site, not the `a.b[i].m()` shape.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w;
  rb ral;
  function int f(); return ral.arr[0].g(); endfunction
endclass
