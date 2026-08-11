// SV-CORPUS-GRAD.13c.2a.2 — CONFIRMED DEFECT (rejects-valid). Expected REJECT today; ACCEPT once GRAMMAR-WELLFORMED.A2.5 lands.
// The `with` continuation of `select_expression`. IEEE 1800-2017 A.2.11:
//   select_expression ::= … | select_expression with ( with_covergroup_expression )
//                             [ matches integer_covergroup_expression ] | cross_identifier | …
// It never fired, because all three of A.2.11's DIRECTLY left-recursive alternatives were dead
// code: PGEN's LR elimination rewrites only the INDIRECT wrapper shape, so a self-reference in
// first position inside a choice was left to the runtime cycle guard, which REJECTS it
// ("💥 Infinite recursion detected in rule 'select_expression'"). The seed won and nothing
// extended it. The fix belongs to the ENGINE, not this grammar (director ruling 2026-08-11): see GRAMMAR-WELLFORMED.A2.5.
// ⛔ THE VERDICT IS NOT THE PROOF. `select_expression` ends in a catch-all arm reaching the general
// expression hierarchy, so the manifest pins the ARM this input must take —
// the arm this row must take once A2.5 lands — and an ACCEPT down any other route FAILS.
// This is the construct the 3 clkmgr_env_cov.sv corpus rows use.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = x with (a == 1); }
  endgroup
endmodule
