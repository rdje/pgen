// SVPP-EXPANSION.5(n) - a FIXED pin (preprocess=1) - a line directive whose parameters are macro usages is expanded, then consumed; must keep ACCEPTING through the pipeline.
// the directive sits inside a procedural block, where a raw line directive the preprocessor left behind has no production (sv2v test/lex/line.sv:9)
// IEEE 1800-2023 22.2 "Macro expansion shall occur within a compiler directive" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:68); IEEE 1800-2017 22.5.1 "Once a text macro name has been defined, it can be used anywhere in the compilation unit where it is defined" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:227); 22.12 Syntax 22-9 (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:713-714)
`define new_line_num 200
`define foo(filename) `"filename.vh`"
module top;
  initial begin
    $display(`__FILE__, `__LINE__);
`line `new_line_num `foo(line) 1
    $display(`__FILE__, `__LINE__);
  end
endmodule
