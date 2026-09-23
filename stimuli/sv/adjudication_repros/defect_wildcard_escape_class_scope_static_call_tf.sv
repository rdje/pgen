// SV-CORPUS-GRAD.30 - a DEFECT pin (REJECT today, both SV profiles) - routed to SV-CORPUS-GRAD.31.
// The CLASS SCOPE of a static method call is an unresolved parameterized class under an open wildcard import - the UVM
// `uvm_config_db#(T)::set(...)` idiom: .27 reached type positions and .28 parameterized class TYPES, but the class-scoped
// call heads (known_unscoped_class_scoped_call_class_identifier, known_unscoped_class_scope_class_identifier) still demand a
// declared class. `ident #( ... ) ::` is never an instantiation and never a package (a package carries no parameter list).
// IEEE 1800-2017 A.8.6 tf_call ::= ps_or_hierarchical_tf_identifier ...; A.9.3 ps_or_hierarchical_tf_identifier ::=
// [ package_scope | class_scope ] tf_identifier; A.1.9 class_scope ::= class_type ::; 26.3 / 26.4.
import q::*;
class d;
  function void f();
    uvm_config_db#(int)::set(null, "a", "b", 1);
  endfunction
endclass
