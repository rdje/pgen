// SV-CORPUS-GRAD.3.44 - CONTROL (the alternative that CANNOT move): IEEE 1800-2017 A.2.7 task_body_declaration
// alternative 1 - `task_identifier ; { tf_item_declaration } { statement_or_null } endtask [ : task_identifier ]` - the
// NON-ported form with its ports declared as tf_item_declaration items inside the body (arm tf_port PRESENT). Typed
// since SV-Slice-25 as {name, items, statements, end_label}; release 1.0.218 leaves it byte-identical. Legal IEEE
// 1364-2005 too (A.2.7 task_declaration with task_item_declaration items).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1123-1130
module m;
  task t;
    input integer a;
    integer x;
    begin
      x = a;
      #1 $display(x);
    end
  endtask : t
  initial t(1);
endmodule
