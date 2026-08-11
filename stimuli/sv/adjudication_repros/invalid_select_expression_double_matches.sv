// SV-CORPUS-GRAD.13c.2a.2 — NEGATIVE reproducer. Expected REJECT, forever.
// IEEE 1800-2017 A.2.11 gives the `with` continuation at most ONE optional tail:
//   select_expression ::= … | select_expression with ( with_covergroup_expression )
//                             [ matches integer_covergroup_expression ] | …
// so `matches 3 matches 4` on a single `with` is not derivable. It is pinned here because
// PGEN-SV-CORPUS-GRAD-0213 FLATTENED that left recursion into `seed ( continuation )*`, and the
// standing hazard of any such rewrite is that the iteration accepts MORE than the recursion did.
// ⛔ Note what is deliberately NOT pinned as invalid: `x with (a) with (b)` and
// `x with (a) matches 3 with (b) matches 4` are both derivable by repeated application of the same
// production, and both must keep parsing. The over-acceptance to guard is the stacked tail, not
// the repeated continuation ([[feedback_sv_strict_lrm_compliance_default]]).
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = x with (a == 1) matches 3 matches 4; }
  endgroup
endmodule
