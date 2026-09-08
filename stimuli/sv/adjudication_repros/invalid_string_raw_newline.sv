// SV-CORPUS-GRAD.3.47 - an INVALID pin: a RAW newline inside a string literal - must keep REJECTING on every profile.
// IEEE 1800-2017 §5.9 (docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101-102): contained in a single line unless the newline is immediately preceded by a backslash; IEEE 1800-2023 §5.9 Syntax 5-3 (reproduced from A.8.8) `quoted_string_item ::= any_ASCII_character except \ or newline or "` (docs/systemverilog/2023/md/section-5-lexical-conventions.md:447-452); IEEE 1364-2005 A.8.8 `Any_ASCII_Characters_except_new_line` (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1218). Through release 1.0.220 the terminal's `[^"\\]` admitted the newline on every profile - an over-acceptance; 0 accepting corpus rows carried one.
module m;
  initial $display("raw
newline");
endmodule
