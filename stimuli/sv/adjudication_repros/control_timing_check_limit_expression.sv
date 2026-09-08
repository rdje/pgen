// SV-CORPUS-GRAD.3.52 - the twin of invalid_timing_check_limit_mintypmax.sv with expression limits: ACCEPT on every profile.
module m;
  reg clk, d;
  specify
    $setuphold (posedge clk, negedge d, 0, 0);
  endspecify
endmodule
