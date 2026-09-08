// SV-CORPUS-GRAD.3.51 - the twin of invalid_hierarchical_delay_value.sv with the ps_identifier A.6.1 derives: ACCEPT on the SV profiles.
module m;
  logic a, b;
  assign #pkg::delay a = b;
endmodule
