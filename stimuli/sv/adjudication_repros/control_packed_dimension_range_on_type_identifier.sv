// SV-CORPUS-GRAD.3.51 - the twin of invalid_packed_dimension_size_on_type_identifier.sv with the range form: ACCEPT.
module m;
  typedef struct packed { logic x; } S;
  S [1:0] s;
endmodule
