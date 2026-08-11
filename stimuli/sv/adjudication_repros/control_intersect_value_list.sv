// Control for `defect_intersect_range_list.sv`: the same `intersect` list WITHOUT a range parses.
// ⚠️ It parses for the WRONG reason — `{ 5, 6 }` is read as a concatenation expression, not as a
// brace-delimited `covergroup_range_list` — which is exactly why the defect beside it exists. This
// control pins the boundary between the two inputs, not the mechanism; the mechanism is in the
// defect file's own comment. A control that is honest about what it does not prove is worth more
// than one that quietly implies more.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = binsof(ca) intersect { 5, 6 }; }
  endgroup
endmodule
