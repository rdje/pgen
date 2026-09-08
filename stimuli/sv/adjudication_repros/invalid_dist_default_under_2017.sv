// SV-CORPUS-GRAD.3.51 - `default` as a dist item under IEEE 1800-2017: `default disable iff 1 dist { [1:2] :/ 3, 2, default :/ 4 };` (slang tests/regression/all.sv) - an EDITION CONTRAST. IEEE 1800-2017 A.1.10 dist_item ::= value_range [ dist_weight ] (docs/systemverilog/2017/md/section-41-data-read-api.md:669) with value_range ::= expression | [ expression : expression ] (docs/systemverilog/2017/md/section-41-data-read-api.md:3044-3047) has no default item; IEEE 1800-2023 A.1.10 dist_item ::= value_range [ dist_weight ] | default :/ expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:636-639, 18.5.3) adds it. The `default disable iff expression_or_dist ;` item itself takes no parentheses in either edition (docs/systemverilog/2017/md/section-41-data-read-api.md:354; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:328).
// REJECTS at `default` under sv_2017 (and under verilog_2005, which has no constraints); the twin control_dist_default_sv2023.sv ACCEPTS under sv_2023.
module m;
  logic x;
  default disable iff 1 dist { [1:2] :/ 3, 2, default :/ 4 };
endmodule
