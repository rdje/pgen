// SVPP-EXPANSION.6f - an INVALID pin (preprocess=1) - an `end_keywords that closes no open region; must keep REJECTING on every profile.
// IEEE 1364-2005 19.11 "Each `begin_keywords directive must be paired with an `end_keywords directive" (docs/verilog/2005/md/section-19-compiler-directives.md: clause 19.11); IEEE 1800-2017 22.14 "the matching `end_keywords directive" (docs/systemverilog/2017/md/section-22-compiler-directives.md:846-870). verilator t_preproc_kwd_bad.v is the shape; before this the stage WARNED and left the line to the parser, which reads it as a directive and accepted the file.
`end_keywords
module top;
endmodule
