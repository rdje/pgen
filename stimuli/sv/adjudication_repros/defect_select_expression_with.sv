// SV-CORPUS-GRAD.13c.2a.2 — CONFIRMED DEFECT (rejects-valid).
// The `with` continuation of `select_expression` never fires. IEEE 1800-2017 A.2.11:
//   select_expression ::= … | select_expression with ( with_covergroup_expression )
//                             [ matches integer_covergroup_expression ] | cross_identifier | …
// Diagnosed mechanism (TOOLBOX §2.2, `--trace-rules bins_selection`): the parse stops at the seed —
//   🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)
//   ✅ Rule 'select_expression' successfully parsed from 127 to 129 (consumed 2 bytes: ' x')
// — the `cross_identifier` arm wins and no `with (…)` continuation is ever applied, while the
// sibling left-recursive `&&` / `||` continuations DO fire
// (see control_select_expression_and.sv). This is the construct 3 opentitan corpus rows use.
// Expected today: REJECT.  Expected once fixed: ACCEPT.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = x with (a == 1); }
  endgroup
endmodule
