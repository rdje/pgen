// SV-CORPUS-GRAD.13e.2 — the one-difference ACCEPT control for
// invalid_v2005_mintypmax_timing_check_limit.sv.
// The identical $setup with its limit written as the plain `expression` A.7.5.2:800 requires.
// It parses, so the rejection above is attributable to the min:typ:max FORM alone and not to
// $setup, to the edge-qualified events, or to the specify block.
module test;
  wire sig1, sig2;
  specify
    $setup(posedge sig1 , negedge sig2 , 0);
  endspecify
endmodule
