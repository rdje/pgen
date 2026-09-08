// SV-CORPUS-GRAD.3.49 - a checker instantiation whose checker is declared in NO compilation unit of this file (another file's design element): BNF-parseable - IEEE 1800-2017 A.4.1.4 checker_instantiation with A.9.3 checker_identifier ::= identifier (section-41-data-read-api.md:1835); whether the name resolves is elaboration semantics outside Annex A, exactly as for module_instantiation (whose module may live in any file). The actuals are sequence syntax, so this can only be a checker instantiation.
module top;
  logic a, b, clk;
  other_file_checker check(@(posedge clk) a[*1:$] ##1 b);
endmodule
