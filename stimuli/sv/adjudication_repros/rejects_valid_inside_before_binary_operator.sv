// SV-CORPUS-GRAD.3.41 - an OPEN DEFECT reproducer - REJECTS today (wrongly); flips to ACCEPT when the leaf lands.
// y = a inside {2'b10, 2'b11} & ~b; - an inside_expression as the LEFT operand of a binary operator (Table 11-2 puts inside with the relational operators, above &)
// IEEE 1800-2017 A.8.3 inside_expression ::= expression inside { open_range_list } docs/systemverilog/2017/md/section-41-data-read-api.md:3043; 11.3.2 Table 11-2 docs/systemverilog/2017/md/section-11-operators-and-expressions.md:294 (row 7: < <= > >= inside dist)
module t;
  logic [1:0] a;
  logic b, y, z;
  always_comb y = a inside {2'b10, 2'b11} & ~b;
  always_comb z = a inside {2'b10, 2'b11} && b;
endmodule
