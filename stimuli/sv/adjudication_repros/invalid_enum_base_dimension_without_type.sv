// SV-CORPUS-GRAD.3.51 - a packed dimension with NO base type on an enum: `typedef enum [0:0] { G_1, G_2 } EnumG;` (sv2v test/core/enum.sv). IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_atom_type [ signing ] | integer_vector_type [ signing ] [ packed_dimension ] | type_identifier [ packed_dimension ] (docs/systemverilog/2017/md/section-41-data-read-api.md:868-873) - the dimension follows a vector type or a type identifier, never stands alone; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856).
// REJECTS at the `[` on both SV profiles; the control `enum logic [0:0]` ACCEPTS.
module m;
  typedef enum [0:0] { G_1, G_2 } EnumG;
endmodule
