// SV-CORPUS-GRAD.3.63 — IEEE 1364-2005 A.7.5.3 / IEEE 1800-2017 A.7.5.3
// `edge_control_specifier ::= edge [ edge_descriptor { , edge_descriptor } ]` with LITERAL brackets
// (§15.4 / §31.5: `edge[01, 0x, x1] clr`). Shape from iverilog/ivtest/ivltests/pr1867332.v; through
// 1.0.230 the parse stopped at the `[`.
module m(input CSB, CL, a);
  specify
    specparam tps = 0.0;
    $setup(posedge CSB, edge[01,0x,x1,1x] CL, tps, a);
  endspecify
endmodule
