// SV-CORPUS-GRAD.3.41 - CONTROL: an inside expression as the RIGHT operand of a binary operator - ACCEPTED before and after.
// y = b & a inside {2'b10, 2'b11}; - the one-difference twin of rejects_valid_inside_before_binary_operator.sv (operand side).
// Until 1.0.212 this derived as the grouped `inside_expression` whose `expr` was the WHOLE chain `b & a`, i.e. `(b & a) inside {..}`
// - a wrong tree on accepted text (Table 11-2: `inside` binds tighter than `&`). Since 1.0.213 it is the flat chain
// [b, &, a, inside{..}] (arm operand_chain>inside), and the grouping is the consumer's precedence pass, as for `b & a == c`.
// IEEE 1800-2017 11.3.2 Table 11-2 docs/systemverilog/2017/md/section-11-operators-and-expressions.md:294 (row 7, line 337)
module t;
  logic [1:0] a;
  logic b, y;
  always_comb y = b & a inside {2'b10, 2'b11};
endmodule
