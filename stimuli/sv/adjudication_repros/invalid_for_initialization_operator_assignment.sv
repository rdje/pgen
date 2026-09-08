// SV-CORPUS-GRAD.3.51 - an OPERATOR assignment as a for-loop initialization: `for (x += 1; x < y; x++)` (sv2v test/core/for_loop_inits.sv). IEEE 1800-2017 A.6.8 for_initialization ::= list_of_variable_assignments | for_variable_declaration { , for_variable_declaration } (docs/systemverilog/2017/md/section-41-data-read-api.md:2336-2340) with A.6.2 variable_assignment ::= variable_lvalue = expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2053); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2321-2325, :2055) - the initialization is a plain `=` assignment, the operator forms belong to for_step.
// REJECTS at the `+` on every profile; the control with `x = 1` ACCEPTS.
module m;
  int x, y;
  initial for (x += 1; x < y; x++) ;
endmodule
