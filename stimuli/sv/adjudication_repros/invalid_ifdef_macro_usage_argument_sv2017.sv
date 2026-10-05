// SVPP-EXPANSION.5(p) - an INVALID pin (preprocess=1), the 1800-2017 reading of the SAME text: the argument is read as written, the guarded text is emitted; must keep REJECTING under sv_2017.
// the byte twin of fixed_ifdef_macro_usage_argument_sv2023.sv
// IEEE 1800-2023 22.2 "Macro expansion shall occur within a compiler directive" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:68); IEEE 1800-2017 22.6 Syntax 22-6 `ifdef text_macro_identifier (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:379-380), with no such sentence
`define ID(x) x
`define HAVE
module m;
`ifndef `ID(HAVE)
  this text is not SystemVerilog
`endif
  wire ok;
endmodule
