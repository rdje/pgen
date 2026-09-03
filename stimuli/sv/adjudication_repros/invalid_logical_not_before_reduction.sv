// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// assign y = !|v; - a unary operator applied to a unary expression
// IEEE 1800-2017 A.8.3 expression ::= ... | unary_operator { attribute_instance } primary section-41-data-read-api.md:272-280; IEEE 1364-2005 A.8.3 identical. t_lint_notredop_bad.v l.12; iverilog's parse-stage vocabulary carries the class.
module t; wire [3:0] v; wire y;
  assign y = !|v;
endmodule
