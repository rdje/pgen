// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// assert property (@(posedge clk) s_always a); - s_always with no range
// IEEE 1800-2017 A.2.10 property_expr section-41-data-read-api.md:1344-1346: always has a range-free arm, s_always has none. t_prop_always_bad.v l.17.
module t; logic clk, a;
  assert property (@(posedge clk) s_always a);
endmodule
