// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// typedef enum logic [1:0][1:0] { A, B } e_t; - two packed dimensions on the enum base
// IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_vector_type [ signing ] [ packed_dimension ] section-41-data-read-api.md:868-871. t_enum_base_bad.v l.25.
module t;
  typedef enum logic [1:0][1:0] { A, B } e_t;
endmodule
