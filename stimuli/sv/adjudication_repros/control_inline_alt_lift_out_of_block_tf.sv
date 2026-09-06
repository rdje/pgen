// SV-CORPUS-GRAD.3.42 - CONTROL: out-of-block method definitions, IEEE 1800-2017 A.2.7
// `function_body_declaration ::= ... | function_data_type_or_implicit [ interface_identifier . | class_scope ]
// function_identifier ...` (and the task twin). ACCEPT before and after. Until release 1.0.214 the INTERFACE form
// (`function void I.h();`, `task I.t();`) emitted six `<invalid_sequence_access>` in this accepting file - alternatives
// 3 and 4 carry no template, so the group's branch 0 wore alternative 1's `{return_type, name, items, statements,
// end_label}` inside the RAW envelope; the CLASS form (`function void C::g();`) was never shadowed (a one-template host
// shadows branch 0 only) and its raw envelope is byte-identical before and after (a Json-returning branch passes
// through unwrapped). Typing these alternatives is SV-CORPUS-GRAD.3.44.
interface I;
  extern function void h();
  extern task t();
endinterface
function void I.h();
endfunction
task I.t();
endtask
class C;
  extern function void g();
  extern task u(int a);
endclass
function void C::g();
endfunction
task C::u(int a);
endtask
