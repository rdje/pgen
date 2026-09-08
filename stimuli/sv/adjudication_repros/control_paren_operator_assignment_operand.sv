// SV-CORPUS-GRAD.3.53 - a parenthesised OPERATOR ASSIGNMENT as an OPERAND of a binary operator: `j = i++ + (i = i - 1);` - IEEE 1800-2017 11.4.2's own example (docs/systemverilog/2017/md/section-11-operators-and-expressions.md:502); A.8.3 expression ::= primary | unary_operator { attribute_instance } primary | inc_or_dec_expression | ( operator_assignment ) | expression binary_operator { attribute_instance } expression | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3034) - the parenthesised assignment IS an expression and so an operand of a binary operator; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2988-2996).
// Through 1.0.223 the grammar admitted `( operator_assignment )` only at the top of an expression (expression_base), never as a chain operand: REJECTED at the `(` after `+`; ACCEPT once paren_operator_assignment_operand is reached from the chain (worklist rows: ispras 11.04.02_01.sv, sv-tests 11.3.6--two_assign_in_expr(-sim).sv).
module m;
  int i, j, a, b, d;
  initial begin
    i = 10;
    j = i++ + (i = i - 1);
    d = ((b += (a += 1) + 1));
    j = (i = i - 1) + 1;
  end
endmodule
