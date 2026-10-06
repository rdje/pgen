// SVPP-EXPANSION.5(r) - an INVALID pin (preprocess=1) - a macro-text line whose backslash is followed by WHITE SPACE before the newline does NOT continue the macro text, on either SV profile; the next lines are source and the parser rejects them.
// IEEE 1800-2017 22.5.1 "the newline character shall be preceded by a backslash ( \ ). The first newline character not preceded by a backslash shall end the macro text" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:135-136); IEEE 1800-2023 22.5.1 "shall be immediately preceded by a backslash" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:149); IEEE 1364-2005 19.3.1 :89-90 the same as 2017.
// The corpus writes this shape at TWO sites, both tests: verilator test_regress/t/t_lint_bsspace_bad.v:10 (verilator's own BSSPACE lint-error test) and Surelog tests/TestMacros/macros.v:5. No design file does. The strict reading stands; no stage option is added.
module m;
`define ASSERT_EQUALS(x, y) \
  initial begin \
    // a one-line comment whose backslash is followed by a tab and spaces \ 	   
    if ((x) != (y)) $display("assert failed %0d != %0d", (x), (y)); \
  end
`ASSERT_EQUALS(1, 1)
endmodule
