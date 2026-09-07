// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.7 task_body_declaration alternative 2 - the PORTED form
// `task_identifier ( [ tf_port_list ] ) ; { block_item_declaration } { statement_or_null } endtask [ : task_identifier ]`.
// ACCEPT before and after on every profile (legal IEEE 1364-2005 A.2.7 too: task_port_list); through 1.0.217 the body
// arrived as the RAW nine-element envelope; since 1.0.218 it is {name, ports, items, statements, end_label} (arm !tf_port).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1131-1135
module m;
  task t(input integer a, output integer y);
    integer x;
    begin
      x = a;
      #1 y = x;
    end
  endtask : t
  integer r;
  initial t(1, r);
endmodule
