// SV-CORPUS-GRAD.3.47 - an INVALID pin under the IEEE 1364-2005 profile: text A.8.8 does not derive - must keep REJECTING there.
// IEEE 1364-2005 §3.6 (docs/verilog/2005/md/section-3-lexical-conventions.md:299): "A string is a sequence of characters enclosed by double quotes and contained on a single line"; A.8.8 (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1218) `string ::= " { Any_ASCII_Characters_except_new_line } "` - Verilog-2005 has no backslash-newline continuation (it is an IEEE 1800 §5.9 feature). Twin: control_string_line_continuation.sv under the SV profiles. Stands for iverilog ivltests/string12.v on the verilog_2005 lane.
module top;
  initial begin
    if ("this matches" == "this\
 matches") $display("PASSED");
  end
endmodule
