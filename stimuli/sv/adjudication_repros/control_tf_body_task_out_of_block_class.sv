// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.7 task_body_declaration alternative 3 - the OUT-OF-BLOCK non-ported
// form `class_scope task_identifier ; { tf_item_declaration } { statement_or_null } endtask` (8.24). ACCEPT before and
// after on both SV profiles; through 1.0.217 the body was the RAW seven-element envelope; since 1.0.218 it is
// {scope, name, items, statements, end_label} with `scope` = the class_scope node and no `ports` key (arm tf_port
// PRESENT: the port is a tf_item_declaration inside the body).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1124-1130
class C;
  extern task u;
endclass
task C::u;
  input int a;
  #a;
endtask : u
