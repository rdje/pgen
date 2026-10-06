// SVPP-EXPANSION.5(t) - a CONTROL pin (preprocess=1), the Verilog half of an EDITION TWIN - IEEE 1364-2005 clause 19 lists no `__FILE__ directive, so under verilog_2005 it is an ordinary macro name and the definition is legal.
// IEEE 1364-2005 19.3.1 "Text macro names may not be the same as compiler directive keywords" (docs/verilog/2005/md/section-19-compiler-directives.md:229) - and `__FILE__ is not one of 1364-2005's (19.1-19.11). The byte twin invalid_define_file_macro_name_under_sv.sv REJECTS under sv_2017 / sv_2023.
`define __FILE__ "unknown"
module top (); initial $display(`__FILE__); endmodule
