// SV-CORPUS-GRAD.30 - a DEFECT pin (REJECT through 1.0.242, both SV profiles) - FIXED by SV-CORPUS-GRAD.31 (release 1.0.243: class_scope's ordered escape alternative + the class_scoped_call_prefix escape arm; class fixed / ACCEPT).
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
