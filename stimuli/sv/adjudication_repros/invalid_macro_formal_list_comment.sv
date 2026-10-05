// SVPP-EXPANSION.5(o) - an INVALID pin (preprocess=1) - a comment between the formals of a text macro definition; no sentence of IEEE 1800-2017 or 1800-2023 admits it; the preprocessor must keep REFUSING it.
// a block comment between two formals (verilator t_preproc.v:277, iverilog ivltests/macro_with_args.v:17, sv2v test/lex/macro_arg_comment.sv:1)
// IEEE 1800-2017 22.5.1 "The formal argument names shall be simple_identifiers, separated by commas and optionally white space" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:143-144; IEEE 1800-2023 docs/systemverilog/2023/txt/section-22-compiler-directives.txt:168-169); 5.3 white space is spaces, tabs, newlines and formfeeds (docs/systemverilog/2017/txt/section-5-lexical-conventions.txt:37); 22.4 "Only white space or a comment may appear on the same line" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:82)
`define ARGPAR(a, /* a comment */ b) (a+b)
module m;
  initial $display(`ARGPAR(1,2));
endmodule
