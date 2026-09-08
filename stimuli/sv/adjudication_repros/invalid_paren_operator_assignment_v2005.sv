// SV-CORPUS-GRAD.3.53 - the parenthesised operator assignment under IEEE 1364-2005, which has no assignment inside an expression: A.8.3 expression ::= primary | unary_operator { attribute_instance } primary | expression binary_operator { attribute_instance } expression | conditional_expression (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1070-1074) - the `( operator_assignment )` alternative is IEEE 1800's (11.4.2). Through 1.0.223 the alternative was unprofiled and `j = (i = i - 1);` PARSED under verilog_2005 - an over-acceptance found by this slice's census, fixed by gating the operand to the SV profiles; REJECT at the `=` inside the parentheses.
module m;
  integer i, j;
  initial j = (i = i - 1);
endmodule
