// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// program p; static task t(); - a lifetime BEFORE the task keyword at program/module level
// IEEE 1800-2017 A.2.7 task_declaration ::= task [ lifetime ] task_body_declaration docs/systemverilog/2017/md/section-41-data-read-api.md:1123 - the lifetime FOLLOWS the keyword; `static` precedes a task only as a class_item_qualifier (A.1.9 class_method docs/systemverilog/2017/md/section-41-data-read-api.md:570)
program p;
  static task t();
  endtask
endprogram
