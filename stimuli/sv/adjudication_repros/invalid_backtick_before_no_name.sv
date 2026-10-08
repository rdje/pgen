// SVPP-EXPANSION.6h - an INVALID pin (preprocess=1) - a backtick that begins neither a compiler directive nor a text-macro usage; must keep REJECTING on every profile.
// IEEE 1800-2017 22.2 "All compiler directives are preceded by the (`) character" (docs/systemverilog/2017/md/section-22-compiler-directives.md:67) - the character introduces a directive NAME - and 22.5.1 Syntax 22-3 `text_macro_usage ::= `text_macro_identifier [ ( list_of_actual_arguments ) ]` (docs/systemverilog/2017/md/section-22-compiler-directives.md:197-201).
// Through 1.0.251 the stage's scan tested only an identifier or a macro-string successor, so a backtick before a line end, white space, a digit or punctuation reached the parser, whose raw-lane residue (compiler_directive -> opaque) accepted the line (the closed loop's generated batches carried `9\W and a bare backtick).
module top;
endmodule
`
