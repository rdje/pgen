// SV-CORPUS-GRAD.13c.2a.2 — CONFIRMED DEFECT (rejects-valid). Expected REJECT today; ACCEPT once GRAMMAR-WELLFORMED.A2.5 lands.
// The `||` continuation of `select_expression` (IEEE 1800-2017 A.2.11). It was dead for the same
// reason as `&&` and `with`: a directly left-recursive alternative inside a choice is rejected by
// the runtime cycle guard, not rewritten by LR elimination.
// ⭐ WHY THE PARENTHESES ARE HERE, AND NOT DECORATION: without them the seed `select_condition`
// swallows the whole remainder, because `intersect`'s LITERAL braces were transcribed as EBNF
// repetition (`.13c.2a.1`), so `{ 1 } || binsof(cb) intersect { 2 }` is absorbed into its
// `covergroup_range_list*`. That masking is what let the previous `&&` control claim, falsely,
// that the sibling continuations fired — see control_select_expression_and.sv. Parenthesizing the
// left operand terminates the seed and hands the rest to the continuation, which is the arm this
// row pins (the arm this row must take once A2.5 lands).
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = ( binsof(ca) intersect { 1 } ) || binsof(cb); }
  endgroup
endmodule
