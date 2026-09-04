// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// task static t(); at program level, and static task t(); as a class method - the two legal spellings
// IEEE 1800-2017 A.2.7 task_declaration ::= task [ lifetime ] task_body_declaration docs/systemverilog/2017/md/section-41-data-read-api.md:1123; A.1.9 class_method ::= { method_qualifier } task_declaration with class_item_qualifier ::= static | protected | local docs/systemverilog/2017/md/section-41-data-read-api.md:570
program p;
  task static t();
  endtask
  task automatic u();
  endtask
endprogram
class c;
  static task t();
  endtask
  virtual task v();
  endtask
endclass
