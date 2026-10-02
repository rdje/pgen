// SV-CORPUS-GRAD.41 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on every profile.
// foreach (x[i]) { solve y before x[i]; } - a solve ... before INSIDE a foreach constraint set
// IEEE 1800-2017 A.1.10 constraint_block_item ::= solve solve_before_list before solve_before_list ; | constraint_expression docs/systemverilog/2017/txt/section-18-constrained-random-value-generation.txt:328-330, while foreach ( ... ) constraint_set (:337) with constraint_set ::= constraint_expression | { { constraint_expression } } (:339-341) admits constraint_expressions alone; IEEE 1800-2023 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:405-407, :415, :423-425
class c;
  rand int x[2];
  rand int y;
  constraint k {
    foreach (x[i]) {
      solve y before x[i];
    }
  }
endclass
