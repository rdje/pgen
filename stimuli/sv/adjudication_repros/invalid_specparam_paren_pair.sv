// ⛔ The control that slice 2 mistook for an exoneration. A plain specparam takes
// `specparam_identifier = constant_mintypmax_expression` (A.7.5) and `(1, 2)` is not one,
// so this rejection is CORRECT — it must be pinned as invalid, or the day some other rule
// is relaxed this text starts parsing and the pass rate "improves".
module m;
  specparam CAP = (1, 2);
endmodule
