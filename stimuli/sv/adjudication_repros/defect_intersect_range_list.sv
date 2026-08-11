// SV-CORPUS-GRAD.13c.2a.1 — CONFIRMED DEFECT (rejects-valid), with an ACCIDENTAL ACCEPT beside it.
// IEEE 1800-2017 A.2.11:
//   select_condition ::= binsof ( bins_expression ) [ intersect { covergroup_range_list } ]
// where `{ }` are LITERAL braces. The grammar transcribed them as EBNF repetition —
//   select_condition := kw_binsof lparen bins_expression rparen ( kw_intersect covergroup_range_list* )?
// — so the braces are gone. `intersect { 5, 6 }` still "works", but only because `{5, 6}` is a
// legal CONCATENATION expression reaching `covergroup_value_range → covergroup_expression`. A
// RANGE is not an expression, so the standard's own §19.6.2 example rejects:
//   cross a, b { ignore_bins ignore = binsof(a) intersect { 5, [1:3] }; }
// ⭐ Same dropped-delimiter class as `.3.8` (a LITERAL `[ ]` read as EBNF optional-grouping), and
// the accidental route means the rule is currently BOTH under- and over-accepting.
// Expected today: REJECT.  Expected once fixed: ACCEPT.
module m;
  bit [2:0] a, b;
  covergroup yy;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb {
      ignore_bins ignore = binsof(ca) intersect { 5, [1:3] };
    }
  endgroup
endmodule
