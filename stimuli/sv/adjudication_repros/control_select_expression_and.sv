// ⛔⛔ THIS FILE IS A MASKING PIN, NOT A CONTROL — corrected in PGEN-SV-CORPUS-GRAD-0213.
// It shipped in PGEN-SV-CORPUS-GRAD-0212 claiming: *"both operands carry the KEYWORD `intersect`,
// so this cannot be slipping through as one ordinary SV expression"* — and that claim was FALSE.
// It never was an ordinary expression; it was the SEED swallowing everything. Dumping the AST
// settles it: exactly one `condition` node and ZERO `and` nodes, with `binsof(cb)` parsed as a
// plain subroutine call and `intersect` as a hierarchical identifier, all absorbed into
// `select_condition`'s `covergroup_range_list*` — because `intersect`'s LITERAL braces were
// transcribed as EBNF repetition (`.13c.2a.1`, still open). The `&&` continuation was dead the
// whole time; this file's ACCEPT is what a dead continuation looks like from the outside.
// ⭐ THE LESSON THAT COST A LEAF: ruling out ONE accidental route is not ruling out the accidental
// route. Read the arm out of the AST — which is why `arm` now exists in MANIFEST.tsv.
// ⭐ WHAT IT IS FOR NOW: it pins the masking itself: `condition,!and` — a `condition` node and ZERO `and` nodes. When `.13c.2a.1` restores
// the literal braces the seed will stop over-consuming, this row will FAIL, and that failure is the
// signal to re-adjudicate it as a genuine `&&` control. Until then the real `&&` proof will be
// defect_select_expression_paren.sv, whose left operand is parenthesized so the seed must stop.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = binsof(ca) intersect { 1 } && binsof(cb) intersect { 2 }; }
  endgroup
endmodule
