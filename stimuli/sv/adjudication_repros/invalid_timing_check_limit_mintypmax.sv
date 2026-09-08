// SV-CORPUS-GRAD.3.52 - a min:typ:max triple as a TIMING CHECK LIMIT: `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator t_setuphold.v:46). IEEE 1800-2017 A.7.5.1 $setuphold_timing_check ::= $setuphold ( reference_event , data_event , timing_check_limit , timing_check_limit [ , … ] ) ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2732-2735) with A.7.5.3 timing_check_limit ::= expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2817) - an expression, and a mintypmax_expression is not one (A.8.3 docs/systemverilog/2017/md/section-41-data-read-api.md:3026); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2709-2712, :2794); IEEE 1364-2005 A.7.5.1 / A.7.5.3 the same.
// REJECTS at the first `:` on every profile; the control with plain limits ACCEPTS.
module m;
  reg clk, d;
  specify
    $setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);
  endspecify
endmodule
