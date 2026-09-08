// SV-CORPUS-GRAD.3.53 - the parenthesised operator assignment at the TOP of an expression, which parsed before and after (its typed AST byte-identical): `j = (i = i - 1);` and a nonblocking assignment whose RHS is `(x = 2)` (verilator t_nba_assign_on_rhs.v) - IEEE 1800-2017 A.8.3 ( operator_assignment ) (docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3034).
module m;
  int i, j, x, y;
  initial begin
    j = (i = i - 1);
    y <= (x = 2);
  end
endmodule
