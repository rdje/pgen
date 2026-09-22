// SV-CORPUS-GRAD.29 - an INVALID pin (must REJECT on both SV profiles, before and after 1.0.241).
// `this[0]`: the implicit class handle is not an array and has no select in any production - IEEE 1800-2017 A.8.4 primary ::= ... | this
// is bare, and the `.29` class_qualifier prefix requires the `.` (implicit_class_handle .) before a hierarchical_identifier.
class c;
  int arr[4];
  int id;
  function void f();
    id = this[0];
  endfunction
endclass
