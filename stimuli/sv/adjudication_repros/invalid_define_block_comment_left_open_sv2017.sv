// SVPP-EXPANSION.5(p) - an INVALID pin (preprocess=1), the 1800-2017 reading of the SAME text: the newline ends the macro text inside a comment; must keep REJECTING under sv_2017.
// the byte twin of fixed_define_multi_line_block_comment_sv2023.sv
// IEEE 1800-2023 22.5.1 "A newline character that is not contained in a triple-quoted string literal or in a block comment and is not preceded by a backslash shall end the macro text" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:150-151), "If the macro text contains a multi-line block comment, the newline characters in the block comment are not required to be preceded by a backslash" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:177-179); IEEE 1800-2017 22.5.1 "The first newline character not preceded by a backslash shall end the macro text" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:135-136), the text "shall not be split across ... Comments" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:230-232)
`define X 1 /* a comment that
continues */ + 2
module m;
  wire [`X:0] w;
endmodule
