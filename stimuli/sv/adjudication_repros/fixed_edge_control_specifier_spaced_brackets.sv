// SV-CORPUS-GRAD.3.63 — the same production with white space and the two-digit descriptors only
// (`edge [01, 10] CL`); a second fixed spelling so the bracket tokens are proven as tokens, not as
// part of the keyword.
module m(input CSB, CL, a);
  specify
    specparam tps = 0.0;
    $setup(posedge CSB, edge [01, 10] CL, tps, a);
  endspecify
endmodule
