// SV-CORPUS-GRAD.3.51 - the twin of invalid_sized_unbased_unsized_literal.sv with the unsized literal A.8.7 derives: ACCEPT on the SV profiles.
module m;
  logic [14:0] a;
  initial a = '1;
endmodule
