// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.6 function_body_declaration alternative 4 - the OUT-OF-BLOCK
// PORTED form `function_data_type_or_implicit class_scope function_identifier ( [ tf_port_list ] ) ;
// { block_item_declaration } { function_statement_or_null } endfunction` (8.24: an extern class method defined
// outside the class). ACCEPT before and after on both SV profiles; through 1.0.217 the body was the RAW eleven-element
// envelope; since 1.0.218 it is {return_type, scope, name, ports, items, statements, end_label} with `scope` = the
// class_scope node and `ports` present (arm !tf_port).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1096-1100; section-8-classes.md (8.24 out-of-block declarations)
class C;
  extern function int g(int a, int b = 2);
endclass
function int C::g(int a, int b = 2);
  int x;
  x = a * b;
  return x;
endfunction : g
