// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// assert #0 (clk); - the LRM's deferred form
// IEEE 1800-2017 A.6.10 section-41-data-read-api.md:2421-2424. isolates the defect to the delay value.
module t (input clk);
  labeled_imas : assert #0 (clk);
endmodule
