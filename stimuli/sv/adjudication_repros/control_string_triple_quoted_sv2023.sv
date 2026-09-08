// SV-CORPUS-GRAD.3.47 - a CONTROL under sv_2023: the IEEE 1800-2023 §5.9 TRIPLE-QUOTED string - must keep ACCEPTING there.
// 1800-2023 §5.9 (docs/systemverilog/2023/md/section-5-lexical-conventions.md:436-452): `triple_quoted_string ::= """ { triple_quoted_string_item | string_escape_seq } """`, an item being any ASCII character except `\` - a bare `"` and a bare newline included. verilator t_display_qqq.v is the corpus row (an EDITION CONTRAST on the 2017 lane: 1800-2017 §5.9 has no such form).
module m;
  initial begin
    $display("""First "quoted"\nsecond\
third
fourth""");
    $display("""simple""");
  end
endmodule
