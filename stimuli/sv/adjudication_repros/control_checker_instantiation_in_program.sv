// SV-CORPUS-GRAD.3.49 - the iverilog ivltests/program5b.v shape: `test dut(foo);` inside a program block. Not a module_instantiation (A.1.7 non_port_program_item admits no module instantiation) but BNF-parseable as an A.4.1.4 checker_instantiation through concurrent_assertion_item (A.2.10), which non_port_program_item admits; whether `test` denotes a module or a checker is resolution semantics (the row's own key, SV-CORPUS-GRAD.8b.3). Through 1.0.222 the has_fact(checker_name) gate rejected it at `test`.
module test(input wire foo);
  initial $display(foo);
endmodule

program main;
  reg foo = 1;
  test dut(foo);
endprogram
