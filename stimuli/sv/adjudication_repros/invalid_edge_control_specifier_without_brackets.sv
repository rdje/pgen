// SV-CORPUS-GRAD.3.63 — an edge control specifier WITHOUT its brackets (`edge 01 CL`): no production in
// either edition (A.7.5.3's brackets are literal). Through 1.0.230 the rule read the brackets as EBNF
// optionality and ACCEPTED this; newly REJECTED by .3.63, and correctly so, on every profile.
module m(input CSB, CL, a);
  specify
    specparam tps = 0.0;
    $setup(posedge CSB, edge 01 CL, tps, a);
  endspecify
endmodule
