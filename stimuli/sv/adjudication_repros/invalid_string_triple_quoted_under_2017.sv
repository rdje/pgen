// SV-CORPUS-GRAD.3.47 - an INVALID pin under sv_2017 and verilog_2005: the 1800-2023 triple-quoted string is no 1800-2017 / 1364-2005 form - must keep REJECTING there.
// IEEE 1800-2017 §5.9 (docs/systemverilog/2017/md/section-32-h-12ab-f001.md:99-102) defines one string form, the double-quoted one; the `"""` form is IEEE 1800-2023 §5.9 (docs/systemverilog/2023/md/section-5-lexical-conventions.md:436-452). Under 2017 `""` is an empty string and the third `"` opens a string that runs to the next `"`, so the text cannot parse. Twin: control_string_triple_quoted_sv2023.sv.
module m;
  initial $display("""simple""");
endmodule
