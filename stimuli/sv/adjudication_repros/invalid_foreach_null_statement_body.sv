// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// foreach (array[i]); - a bare ; as the loop body
// IEEE 1800-2017 A.6.8 loop_statement section-41-data-read-api.md:2325-2334: foreach takes statement, the other loops statement_or_null; A.6.4 statement_or_null section-41-data-read-api.md:2081. t_foreach_nindex_bad.v l.12, t_foreach_type_bad.v l.19 (and t_foreach_bad.v).
module t; int array[2];
  initial foreach (array[i]);
endmodule
