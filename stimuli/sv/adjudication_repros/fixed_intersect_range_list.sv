// SV-CORPUS-GRAD.13c.2a.1 — ✅ FIXED by `.13c.2e` (`PGEN-SV-CORPUS-GRAD-0214`). Expect ACCEPT.
// IEEE 1800-2017 A.2.11:
//   select_condition ::= binsof ( bins_expression ) [ intersect { covergroup_range_list } ]
// where `{ }` are LITERAL braces. The grammar transcribed them as EBNF repetition —
//   select_condition := kw_binsof lparen bins_expression rparen ( kw_intersect covergroup_range_list* )?
// — so the braces were gone. `intersect { 5, 6 }` still "worked", but only because `{5, 6}` is a
// legal CONCATENATION expression reaching `covergroup_value_range → covergroup_expression`. A
// RANGE is not an expression, so the standard's own §19.6.2 example — this file — REJECTED.
// ⭐ Same dropped-delimiter class as `.3.8` (a LITERAL `[ ]` read as EBNF optional-grouping) and
// as ledger `SV-0049`, which is the same defect one clause away (`expression_or_dist`'s
// `dist { dist_list }`). The rule was BOTH under- and over-accepting; both legs are closed.
// ⛔ WHAT THE ARM PINS, and why a verdict alone would not do. `condition>range` says the `[1:3]`
// arrived as a genuine `covergroup_value_range` RANGE inside the select condition — not as some
// expression that merely happened to parse. `!condition>concat` is the REVERT guard: restoring
// `covergroup_range_list*` brings the concatenation route back, and this row would go green on
// verdict alone if the arm did not forbid it.
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
