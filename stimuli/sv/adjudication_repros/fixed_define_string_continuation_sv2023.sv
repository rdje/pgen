// SVPP-EXPANSION.5(u) - a FIXED pin (preprocess=1), the 2023 half of an EDITION TWIN - a backslash-newline INSIDE a string literal of the macro text: both characters are omitted, the literal reads "two lines"; must keep ACCEPTING under sv_2023.
// IEEE 1800-2023 22.5.1 "An exception to the previous sentence is that if the backslash-newline character sequence occurs in the middle of a string literal, both the backslash and the newline characters are omitted in the expanded macro (see 5.9)" (docs/systemverilog/2023/md/section-22-compiler-directives.md:182-184). The byte twin invalid_define_string_continuation_sv2017.sv REJECTS under sv_2017 / verilog_2005, whose text replaces the continuation with a newline (docs/systemverilog/2017/md/section-22-compiler-directives.md:171-175) and so yields a two-line literal the parser refuses (5.9).
`define MSG "two \
lines"
module top ();
initial $display(`MSG);
endmodule
