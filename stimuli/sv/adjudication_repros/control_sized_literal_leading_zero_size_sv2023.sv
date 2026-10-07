// SV-CORPUS-GRAD.47 - a CONTROL pin - text IEEE 1800-2023 derives (size ::= unsigned_number), ACCEPTED under sv_2023; its sv_2017 / verilog_2005 twin must keep REJECTING.
// {01'b0, 7'd3} / 007'h5 + 02'd1 - sized literals whose SIZE begins with 0, in three bases
// IEEE 1800-2023 5.7 Syntax 5-2 size ::= unsigned_number, unsigned_number ::= decimal_digit { _ | decimal_digit } (docs/systemverilog/2023/md/section-5-lexical-conventions.md:213, :222) and A.8.7 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3200) - the 2023 edition defines no non_zero_unsigned_number; its 5.7.1 keeps the size 'a nonzero unsigned decimal number' (docs/systemverilog/2023/md/section-5-lexical-conventions.md:258), so a ZERO size stays refused
module m;
  wire [7:0] x, y;
  assign x = {01'b0, 7'd3};
  assign y = 007'h5 + 02'd1;
endmodule
