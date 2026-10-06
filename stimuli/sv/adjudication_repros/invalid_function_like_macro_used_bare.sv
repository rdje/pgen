// SVPP-EXPANSION.6f - an INVALID pin (preprocess=1) - a function-like macro used without its argument list; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 "For a macro with arguments, the parentheses are always required in the macro call, even if all the arguments have defaults" (docs/systemverilog/2017/md/section-22-compiler-directives.md:224-225). sv2v test/error/macro_unapplied.sv is the shape; before this the usage was kept as written and the opaque residue accepted the line.
`define MACRO(a) a
`MACRO asdf
module top; endmodule
