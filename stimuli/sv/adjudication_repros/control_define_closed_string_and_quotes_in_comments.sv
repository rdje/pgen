// SVPP-EXPANSION.5(s) - a CONTROL pin (preprocess=1) - a quote inside a comment in macro text is not a string delimiter, and a complete string literal in macro text is fine; must keep ACCEPTING on every profile.
// IEEE 1800-2017 22.5.1 (docs/systemverilog/2017/md/section-22-compiler-directives.md:275-281): the refusal is for a string literal the macro text leaves OPEN at its end (invalid_define_macro_text_split_across_string.sv), not for one it closes.
`define WHOLE "a whole string"
`define Q 1 // a " in a one-line comment
`define R 2 /* a " in a block comment */
module top ();
initial $display(`WHOLE, `Q, `R);
endmodule
