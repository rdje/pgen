// SV-CORPUS-GRAD.3.52 - a unary operator applied to a UNARY EXPRESSION: `!(-~c == 'h22)` (verilator t_constraint_operators.v:52). IEEE 1800-2017 A.8.3 expression ::= primary | unary_operator { attribute_instance } primary | inc_or_dec_expression | ( operator_assignment ) | expression binary_operator { attribute_instance } expression | … (docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3034) - a unary operator takes a PRIMARY, and `~c` is an expression, not a primary (A.8.4 primary docs/systemverilog/2017/md/section-41-data-read-api.md:3122); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2988-2996, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079); IEEE 1364-2005 A.8.3 expression ::= primary | unary_operator { attribute_instance } primary | … the same shape. `-(~c)` is the derivable spelling.
// REJECTS at the `~` on every profile; the control `-(~c)` ACCEPTS.
module m;
  reg [7:0] c;
  wire [7:0] d;
  assign d = -~c;
endmodule
