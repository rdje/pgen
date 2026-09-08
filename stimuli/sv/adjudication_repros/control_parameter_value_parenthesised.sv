// SV-CORPUS-GRAD.3.51 - the twin of invalid_parameter_value_without_parentheses.sv with the parentheses A.4.1.1 requires: ACCEPT on every profile.
module m;
  m1 #(PAR) u1();
  mreal #(1.2) mr ();
endmodule
