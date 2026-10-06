// SVPP-EXPANSION.5(r) - a CONTROL pin (preprocess=1) - a one-line comment inside macro text whose line ends in backslash-newline CONTINUES the macro text; the comment is not substituted; must keep ACCEPTING on both SV profiles.
// IEEE 1800-2023 22.5.1 "If a one-line comment is followed by a backslash and newline character, the comment ends before the backslash and the macro text continues on the next line" (docs/systemverilog/2023/md/section-22-compiler-directives.md:200-201); 1800-2017 22.5.1 docs/systemverilog/2017/md/section-22-compiler-directives.md:171-172 (the newline preceded by a backslash continues the text) + :187-188 (the comment is not substituted) read together.
// The backslash is IMMEDIATELY before the newline here - the licensed shape (Surelog tests/TestMacros/macros.v:5 writes a tab and spaces after its backslash, which no edition licenses; see invalid_define_backslash_trailing_space.sv).
module m;
`define ASSERT_EQUALS(x, y) \
  initial begin \
    // a one-line comment inside the macro text \
    if ((x) != (y)) $display("assert failed %0d != %0d", (x), (y)); \
  end
`ASSERT_EQUALS(1, 1)
endmodule
