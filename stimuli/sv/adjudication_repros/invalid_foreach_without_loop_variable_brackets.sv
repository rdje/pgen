// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// foreach (array) $display(1); - no bracketed loop-variable list
// IEEE 1800-2017 A.6.8 foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement section-41-data-read-api.md:2334; 12.7.3 section-12-procedural-programming-statements.md:1045. t_foreach_bad.v l.14.
module t; int array[2];
  initial foreach (array) $display(1);
endmodule
