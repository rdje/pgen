// SV-CORPUS-GRAD.32 - an IDENTITY CONTROL - ACCEPT before and after release 1.0.245, the typed AST byte-identical (both SV profiles).
// A nested class named through its outer class as a function-body local (`outer::inner h;`). Measured on 1.0.244: `block_data_type`
// selects branch 9/18 (`scoped_block_type_identifier`, through `class_scope_plain`), not the provisional arm.
// IEEE 1800-2017 A.2.2.1 data_type; A.1.9 class_scope ::= class_type ::; 8.23 nested classes
class outer;
  class inner;
    int v;
  endclass
  function void f();
    outer::inner h;
    h = null;
  endfunction
endclass
