// SVPP-EXPANSION.5(u) - an INVALID pin (preprocess=1), the 2017 half of an EDITION TWIN - a backslash-newline INSIDE a string literal of the macro text: 1800-2017 replaces it with a newline, so the expanded literal spans two lines and the PARSER rejects it (5.9: a string literal is one line); the same bytes ACCEPT under sv_2023 (fixed_define_string_continuation_sv2023.sv).
// IEEE 1800-2017 22.5.1 "The newline character preceded by a backslash shall be replaced in the expanded macro with a newline character" (docs/systemverilog/2017/md/section-22-compiler-directives.md:171-175) — no exception for a string literal in this edition; IEEE 1364-2005 19.3.1 the same.
`define MSG "two \
lines"
module top ();
initial $display(`MSG);
endmodule
