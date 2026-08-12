// Control for `fixed_intersect_range_list.sv`: the same `intersect` list WITHOUT a range.
// ⭐ IT NOW PARSES FOR THE RIGHT REASON, and that change is the point of this file's history.
// Before `.13c.2e` this row passed while `{ 5, 6 }` was read as ONE concatenation expression —
// the accidental route that made the defect beside it possible — so the control pinned the
// boundary between the two inputs and explicitly refused to claim the mechanism. With the
// literal braces restored, `{ 5, 6 }` is a two-item `covergroup_range_list`, and the arm now
// pins the mechanism as well: `!concat` fails the moment anyone restores `covergroup_range_list*`,
// because that spelling can only reach this text through the concatenation route again.
// ⛔ The lesson is kept rather than deleted: a control that is honest about what it does not
// prove is worth more than one that quietly implies more — and when the missing proof arrives,
// the honest control is the one that can be upgraded instead of rewritten.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = binsof(ca) intersect { 5, 6 }; }
  endgroup
endmodule
