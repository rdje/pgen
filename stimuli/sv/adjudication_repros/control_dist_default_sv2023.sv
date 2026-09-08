// SV-CORPUS-GRAD.3.51 - the 2023 twin of invalid_dist_default_under_2017.sv: IEEE 1800-2023 A.1.10 dist_item ::= value_range [ dist_weight ] | default :/ expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:636-639) - ACCEPT under sv_2023.
module m;
  logic x;
  default disable iff 1 dist { [1:2] :/ 3, 2, default :/ 4 };
endmodule
