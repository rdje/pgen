// SV-CORPUS-GRAD.3.51 - the twin of invalid_hierarchical_type_reference.sv with the package scope A.2.2.1 derives: ACCEPT.
module m;
  localparam type t = pkg::rq_t;
endmodule
