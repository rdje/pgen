// SV-CORPUS-GRAD.13c.2a.3 — CONFIRMED DEFECT (rejects-valid).
// The parenthesized alternative of `select_expression` never fires. IEEE 1800-2017 A.2.11:
//   select_expression ::= … | ( select_expression ) | …
// ⛔ The identical text WITHOUT the parentheses parses (control_select_expression_and.sv), and
// parentheses around an operand carrying no `intersect` parse too — because then the whole thing
// is an ordinary SV expression matched by the catch-all `cross_set_expression` arm. `intersect` is
// a keyword, so it forces the real alternative to be taken, and there the parentheses are refused.
// Expected today: REJECT.  Expected once fixed: ACCEPT.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = ( binsof(ca) intersect { 1 } ) && binsof(cb); }
  endgroup
endmodule
