// SV-CORPUS-GRAD.3.41 - FIXED (release 1.0.213): an inside expression as the LEFT operand of a binary operator ACCEPTS.
// y = a inside {2'b10, 2'b11} & ~b; - Table 11-2 (11.3.2) puts `inside` on the relational row, above `&` and `&&`; A.8.3 derives
// `expression binary_operator expression` with `inside_expression ::= expression inside { open_range_list }` as the left side.
// Until 1.0.212 the grouped `inside_expression` sat only at the top of `expression`, so the chain could not continue past the `}`
// (rejected at the `&`, furthest_position=630). Since 1.0.213 `inside { ... }` is a CHAIN ELEMENT of expression_base's operand
// chain: [a, inside{..}, &, ~b] and [a, inside{..}, &&, b] - precedence stays the consumer's, as for every other operator.
// IEEE 1800-2017 A.8.3 docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3043; 11.3.2 Table 11-2
// docs/systemverilog/2017/md/section-11-operators-and-expressions.md:294 (row 7 `< <= > >= inside dist`, line 337)
module t;
  logic [1:0] a;
  logic b, y, z;
  always_comb y = a inside {2'b10, 2'b11} & ~b;
  always_comb z = a inside {2'b10, 2'b11} && b;
endmodule
