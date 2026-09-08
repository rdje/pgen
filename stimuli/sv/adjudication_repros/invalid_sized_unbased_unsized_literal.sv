// SV-CORPUS-GRAD.3.51 - a SIZE on an unbased unsized literal: `a = 15'1;` (iverilog ivtest/ivltests/pr3515542.v). IEEE 1800-2017 A.8.7 unbased_unsized_literal ::= '0 | '1 | 'z_or_x (docs/systemverilog/2017/md/section-41-data-read-api.md:3290) takes no size, and decimal_number ::= unsigned_number | [ size ] decimal_base unsigned_number | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:3253-3258) needs a base after the size; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3231, :3188); IEEE 1364-2005 has no unbased unsized literal at all.
// REJECTS at the `'` after `15` on every profile; the control `a = '1;` ACCEPTS on the SV profiles.
module m;
  logic [14:0] a;
  initial a = 15'1;
endmodule
