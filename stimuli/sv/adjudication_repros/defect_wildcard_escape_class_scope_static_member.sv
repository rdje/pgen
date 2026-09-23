// SV-CORPUS-GRAD.30 - a DEFECT pin (REJECT today, both SV profiles) - routed to SV-CORPUS-GRAD.31.
// A static MEMBER of an unresolved parameterized class, read and assigned through its class scope under an open wildcard
// import (IEEE 1800-2017 A.8.4 primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select, class_qualifier
// ::= [ local :: ] [ implicit_class_handle . | class_scope ]; A.8.5 variable_lvalue).
import q::*;
class d;
  function void f();
    uvm_config_db#(int)::count = 1;
    x = uvm_config_db#(int)::count;
  endfunction
endclass
