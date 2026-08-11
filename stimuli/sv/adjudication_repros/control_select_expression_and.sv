// Control for `defect_select_expression_with.sv`: the SIBLING left-recursive continuations of
// `select_expression` do fire, so the defect is the `with` arm and not LR elimination in general.
// ⛔ Both operands carry `intersect`, which is a KEYWORD — so this cannot be slipping through as
// one ordinary SV expression via the catch-all `cross_set_expression → covergroup_expression →
// expression` arm. A control has to rule out the accidental route, or it is not a control:
// `binsof(ca) && binsof(cb)` also parses, and proves nothing, because it is a legal expression.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = binsof(ca) intersect { 1 } && binsof(cb) intersect { 2 }; }
  endgroup
endmodule
