// SV-CORPUS-GRAD.47 - an INVALID pin - the 1800-2017 / 1364-2005 half of the edition pair; must keep REJECTING on sv_2017 and verilog_2005.
// {01'b0, 7'd3} / 007'h5 + 02'd1 - the same bytes under the older editions: the size begins with a non-zero digit
// IEEE 1800-2017 5.7 Syntax 5-2 size ::= non_zero_unsigned_number ::= non_zero_decimal_digit { _ | decimal_digit } (docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:152-153, :170) and A.8.7 the same (docs/systemverilog/2017/md/section-41-data-read-api.md:3265); 5.7.1 'It shall be specified as a nonzero unsigned decimal number' (docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:201); IEEE 1364-2005 3.5 size ::= non_zero_unsigned_number (docs/verilog/2005/md/section-3-lexical-conventions.md:109), 3.5.1 the same prose (docs/verilog/2005/md/section-3-lexical-conventions.md:149), A.8.7 (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1195)
module m;
  wire [7:0] x, y;
  assign x = {01'b0, 7'd3};
  assign y = 007'h5 + 02'd1;
endmodule
