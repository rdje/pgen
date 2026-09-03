// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// assert property (@(posedge clk) ##1 not val); - a property operator where a sequence_expr is required
// IEEE 1800-2017 A.2.10 sequence_expr ::= cycle_delay_range sequence_expr ... section-41-data-read-api.md:1404-1414; not is a property_expr arm section-41-data-read-api.md:1324. t_property_sexpr_bad.v l.20.
module t; logic clk, val;
  assert property (@(posedge clk) ##1 not val);
endmodule
