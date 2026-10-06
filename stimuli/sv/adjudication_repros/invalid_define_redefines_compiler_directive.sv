// SVPP-EXPANSION.5(t) - an INVALID pin (preprocess=1) - a compiler directive redefined as a macro name; must keep REJECTING on every profile (`define is a directive of every edition).
// IEEE 1800-2017 22.5.1 "All compiler directives shall be considered predefined macro names; it shall be illegal to redefine a compiler directive as a macro name" (docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140); IEEE 1800-2023 22.5.1 (docs/systemverilog/2023/md/section-22-compiler-directives.md:152-153); IEEE 1364-2005 19.3.1 (docs/verilog/2005/md/section-19-compiler-directives.md:100-101, and :229 "Text macro names may not be the same as compiler directive keywords").
// Before this `define define "illegal" and `define include 1 were accepted (sv-tests chapter-22/22.5.1--define-expansion_23.sv, keyed should_fail, ACCEPTED under the pipeline; verilator t_pp_defkwd_bad.v; sv2v test/error/macro_illegal_name.sv).
`define define "illegal"
module top (); endmodule
