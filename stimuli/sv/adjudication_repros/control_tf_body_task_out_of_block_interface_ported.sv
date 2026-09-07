// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.7 task_body_declaration alternative 4 - the OUT-OF-BLOCK PORTED
// form `interface_identifier . task_identifier ( [ tf_port_list ] ) ; { block_item_declaration } { statement_or_null }
// endtask` (25.7: an extern interface method defined outside the interface). ACCEPT before and after on both SV
// profiles; through 1.0.217 the body was the RAW ten-element envelope; since 1.0.218 it is {scope, name, ports, items,
// statements, end_label} with `scope` = the pair [<interface_identifier>, {kind: "dot"}] (arm !tf_port).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1131-1135
interface I;
  extern task t(input int a);
endinterface
task I.t(input int a);
  int x;
  x = a;
  #x;
endtask : t
