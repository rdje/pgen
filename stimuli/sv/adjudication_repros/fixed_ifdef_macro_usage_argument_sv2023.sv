// SVPP-EXPANSION.5(p) - a FIXED pin (preprocess=1), the 1800-2023 reading: a macro usage as the argument of ifndef is expanded first; must keep ACCEPTING under sv_2023.
// ifndef whose argument is a macro usage naming a defined symbol; the guarded text is not SystemVerilog (verilator t_preproc.v:423)
// IEEE 1800-2023 22.2 "Macro expansion shall occur within a compiler directive" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:68); IEEE 1800-2017 22.6 Syntax 22-6 `ifdef text_macro_identifier (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:379-380), with no such sentence
`define ID(x) x
`define HAVE
module m;
`ifndef `ID(HAVE)
  this text is not SystemVerilog
`endif
  wire ok;
endmodule
