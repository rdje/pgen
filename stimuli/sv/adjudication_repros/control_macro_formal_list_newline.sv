// SVPP-EXPANSION.5(o) - a CONTROL pin (preprocess=1) - the same formal list split by a NEWLINE, which IS white space (5.3); must keep ACCEPTING through the pipeline.
// formals on two lines, no comment
// IEEE 1800-2017 22.5.1 "The formal argument names shall be simple_identifiers, separated by commas and optionally white space" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:143-144; IEEE 1800-2023 docs/systemverilog/2023/txt/section-22-compiler-directives.txt:168-169); 5.3 white space is spaces, tabs, newlines and formfeeds (docs/systemverilog/2017/txt/section-5-lexical-conventions.txt:37); 22.4 "Only white space or a comment may appear on the same line" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:82)
`define ARGPAR(a,
  b) (a+b)
module m;
  initial $display(`ARGPAR(1,2));
endmodule
