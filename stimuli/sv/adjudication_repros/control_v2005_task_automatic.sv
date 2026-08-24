// SV-CORPUS-GRAD.13e.7 — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.2.7 task_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:275) ::= task [ automatic ] task_identifier ;
// WHY: the sibling of v2005_function_automatic; two controls because `lifetime` is referenced from both and a gate could reach one
module test;
  task automatic t;
  endtask
endmodule
