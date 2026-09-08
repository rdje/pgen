// SV-CORPUS-GRAD.3.52 - the twin of invalid_parenthesised_type_declaration.sv without the parentheses: ACCEPT.
module m;
  typedef enum {A, B} states_t;
  states_t state1;
endmodule
