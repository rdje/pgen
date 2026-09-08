// SV-CORPUS-GRAD.3.51 - a NEGATIVE enum name index: `enum {ndef[-1]} ndef1;` (iverilog ivtest/ivltests/pr3366217d.v). IEEE 1800-2017 A.2.2.1 enum_name_declaration ::= enum_identifier [ [ integral_number [ : integral_number ] ] ] [ = constant_expression ] (docs/systemverilog/2017/md/section-41-data-read-api.md:875-878) with A.8.7 integral_number ::= decimal_number | octal_number | binary_number | hex_number (docs/systemverilog/2017/md/section-41-data-read-api.md:3240-3244) - an unsigned literal, no sign; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:858-861, :3180-3184).
// REJECTS at the `-` on both SV profiles; the control `ndef[1]` ACCEPTS.
module m;
  enum {ndef[-1]} ndef1;
endmodule
