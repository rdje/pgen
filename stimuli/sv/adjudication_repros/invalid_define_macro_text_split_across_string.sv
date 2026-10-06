// SVPP-EXPANSION.5(s) - an INVALID pin (preprocess=1) - macro text split across a STRING LITERAL: the standard's own illegal example; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 "The text specified for macro text shall not be split across the following lexical tokens: Comments, Numbers, String literals, Identifiers, Keywords, Operators" (docs/systemverilog/2017/md/section-22-compiler-directives.md:275-281) and "The following is illegal syntax because it is split across a string" (docs/systemverilog/2017/md/section-22-compiler-directives.md:295-297); IEEE 1800-2023 22.5.1 (docs/systemverilog/2023/md/section-22-compiler-directives.md:307, :320-322); IEEE 1364-2005 19.3.1 (docs/verilog/2005/md/section-19-compiler-directives.md:172, :192, :214-215).
// Before this the definition was accepted and the use site glued one string literal (sv-tests chapter-22/22.5.1--define-expansion_21.sv, keyed should_fail, ACCEPTED under the pipeline).
`define first_half "start of string
module top ();
initial $display(`first_half end of string");
endmodule
