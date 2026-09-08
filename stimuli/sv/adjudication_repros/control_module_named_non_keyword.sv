// SV-CORPUS-GRAD.3.51 - the twin of invalid_reserved_word_medium_as_module_name.sv with a non-reserved name: ACCEPT on every profile.
module top;
  wire o;
  assign o = med.c;
  med u1();
endmodule
module med();
  wire c;
endmodule
