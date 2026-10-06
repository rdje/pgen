// SVPP-EXPANSION.5(t) - an INVALID pin (preprocess=1), the SV half of an EDITION TWIN - `__FILE__ is an IEEE 1800 directive (22.13), so `define __FILE__ is refused under the SV profiles.
// IEEE 1800-2017 22.5.1 (docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140); 22.13 `__FILE__ and `__LINE__ (docs/systemverilog/2017/md/section-22-compiler-directives.md:823-828). The byte twin control_define_file_macro_name_under_verilog_2005.sv ACCEPTS under verilog_2005, whose IEEE 1364-2005 19 lists no such directive.
`define __FILE__ "unknown"
module top (); initial $display(`__FILE__); endmodule
