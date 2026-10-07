// SV-CORPUS-GRAD.47 - a CONTROL pin - sized literals every edition derives, ACCEPTED on every profile through the strict terminal.
// {1'b0, 7'd3} / 16'hFF + 'h5 + 10'd0 / 100'b1 - every size the three editions derive (a non-zero first digit, or no size at all)
// IEEE 1800-2017 5.7 Syntax 5-2 size ::= non_zero_unsigned_number ::= non_zero_decimal_digit { _ | decimal_digit } (docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:152-153, :170) and A.8.7 the same (docs/systemverilog/2017/md/section-41-data-read-api.md:3265); 5.7.1 'It shall be specified as a nonzero unsigned decimal number' (docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:201); IEEE 1800-2023 5.7 Syntax 5-2 size ::= unsigned_number, unsigned_number ::= decimal_digit { _ | decimal_digit } (docs/systemverilog/2023/md/section-5-lexical-conventions.md:213, :222) and A.8.7 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3200) - the 2023 edition defines no non_zero_unsigned_number; its 5.7.1 keeps the size 'a nonzero unsigned decimal number' (docs/systemverilog/2023/md/section-5-lexical-conventions.md:258), so a ZERO size stays refused; IEEE 1364-2005 3.5 size ::= non_zero_unsigned_number (docs/verilog/2005/md/section-3-lexical-conventions.md:109), 3.5.1 the same prose (docs/verilog/2005/md/section-3-lexical-conventions.md:149), A.8.7 (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1195)
module m;
  wire [7:0] x;
  wire [15:0] w;
  wire [99:0] big;
  assign x = {1'b0, 7'd3};
  assign w = 16'hFF + 'h5 + 10'd0;
  assign big = 100'b1;
endmodule
