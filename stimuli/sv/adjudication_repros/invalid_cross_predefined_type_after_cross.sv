// SV-CORPUS-GRAD.3.29 — an INVALID pin: CrossQueueType used OUTSIDE the cross that predefines it.
// IEEE 1800-2017 §19.6.1.3 (docs/systemverilog/2017/md/section-19-functional-coverage.md:1307-1308):
// "The scope of the type names is the cross itself and the types are not accessible outside of this
// scope." This module declares the name nowhere, so the parser's sound store gate is RIGHT to reject
// `CrossQueueType q;` here — before the fix (no fact anywhere) and after it (the fact lives in the
// `cross` scope cross_body_close popped). WHY: an implementation that emitted the two facts into the
// enclosing scope instead of a cross-owned one would start ACCEPTING this file — over-acceptance.
module m;
  logic [31:0] a, b;
  covergroup cg;
    coverpoint a;
    coverpoint b;
    aXb : cross a, b { }
  endgroup
  CrossQueueType q;
endmodule
