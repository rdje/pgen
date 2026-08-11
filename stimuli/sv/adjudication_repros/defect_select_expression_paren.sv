// SV-CORPUS-GRAD.13c.2a.3 — CONFIRMED DEFECT (rejects-valid). Expected REJECT today; ACCEPT once GRAMMAR-WELLFORMED.A2.5 lands.
// The parenthesized alternative of `select_expression`. IEEE 1800-2017 A.2.11:
//   select_expression ::= … | ( select_expression ) | select_expression && select_expression | …
// ⭐ The paren arm was never the defect — it fired all along (`( binsof(ca) intersect {1} )` alone
// parsed). What did not fire was the `&&` CONTINUATION that has to extend it, so a parenthesized
// operand looked inert whenever anything followed it. One fix closes `.13c.2a.2` and this leaf,
// because both are the same dead directly-left-recursive alternative.
// ⛔ Parentheses around an operand carrying no `intersect` prove nothing — that is an ordinary SV
// expression reaching the catch-all `cross_set_expression` arm. `intersect` is a keyword, so it
// forces the real alternative; and the manifest additionally pins the ARM
// (the arm this row must take once A2.5 lands) so an accept down the accidental route still FAILS.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = ( binsof(ca) intersect { 1 } ) && binsof(cb); }
  endgroup
endmodule
