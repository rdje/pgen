// SV-CORPUS-GRAD.13c.2a.3 — CONFIRMED DEFECT (rejects-valid). Expected REJECT today; ACCEPT once GRAMMAR-WELLFORMED.A2.5 lands.
// The parenthesized alternative of `select_expression`. IEEE 1800-2017 A.2.11:
//   select_expression ::= … | ( select_expression ) | select_expression && select_expression | …
// ⭐ The paren arm was never the defect — it fired all along (`( binsof(ca) intersect {1} )` alone
// parsed). What did not fire was the `&&` CONTINUATION that has to extend it, so a parenthesized
// operand looked inert whenever anything followed it. One fix closes `.13c.2a.2` and this leaf,
// because both are the same dead directly-left-recursive alternative.
// ⛔ Parentheses around an operand carrying no `intersect` prove nothing — that is an ordinary SV
// expression reaching the catch-all `cross_set_expression` arm.
// ⛔⛔ CORRECTED BY `.13c.2e` (`PGEN-SV-CORPUS-GRAD-0214`). This file used to continue: *"`intersect`
// is a keyword, so it forces the real alternative"*. That was FALSE, and measured so. Until
// `.13c.2e` restored `select_condition`'s LITERAL LRM braces, `intersect` forced nothing: the
// brace-free `covergroup_range_list*` read `{ 1 }` as a CONCATENATION and swallowed everything
// after it, so a keyword-bearing operand was exactly as inert as a keyword-free one — measured on
// the sibling `control_select_expression_and.sv`, whose AST held ONE `condition` and ZERO `and`
// nodes. What was load-bearing here was the PARENTHESES, which stop the seed by construction.
// ⇒ The keyword argument is retired; the paren argument stands. The braces now stop the seed too,
// which is why the sibling could be promoted from a masking pin to a real `&&` control.
// The manifest additionally pins the ARM, so an accept down an accidental route still FAILS.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = ( binsof(ca) intersect { 1 } ) && binsof(cb); }
  endgroup
endmodule
