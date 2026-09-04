// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// randsequence code block { automatic int x = 1; ... } - A.6.12 rs_code_block is a procedural context, footnote 10 does not reach it
// IEEE 1800-2017 Annex A footnote 10 docs/systemverilog/2017/md/section-41-data-read-api.md:3503-3505: in a data_declaration that is not within a procedural context, it shall be illegal to use the automatic keyword; A.6.12 rs_code_block ::= { { data_declaration } { statement_or_null } }
module t; initial begin randsequence(main) main : first second; first : { automatic int x = 1; $display(x); }; second : { int y; y = 2; }; endsequence end endmodule
