// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.6 function_body_declaration alternative 3 - the OUT-OF-BLOCK
// non-ported form `function_data_type_or_implicit interface_identifier . function_identifier ; { tf_item_declaration }
// { function_statement_or_null } endfunction` (8.24 / 25.7: an extern method of an interface defined outside it).
// ACCEPT before and after on both SV profiles; through 1.0.217 the body was the RAW eight-element envelope (and until
// 1.0.214 carried six <invalid_sequence_access>, SV-CORPUS-GRAD.3.42); since 1.0.218 it is {return_type, scope, name,
// items, statements, end_label} with `scope` = the pair [<interface_identifier>, {kind: "dot"}] and no `ports` key
// (arm tf_port PRESENT: the port is a tf_item_declaration inside the body).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1091-1095
interface I;
  extern function int h;
endinterface
function int I.h;
  input int a;
  return a + 1;
endfunction : h
