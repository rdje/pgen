// SV-CORPUS-GRAD.3.51 - a PACKAGE-SCOPED enum base type: `typedef enum pkg1::uint8_t { ... }` (verilator t_typedef_package.v). IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_atom_type [ signing ] | integer_vector_type [ signing ] [ packed_dimension ] | type_identifier [ packed_dimension ] (docs/systemverilog/2017/md/section-41-data-read-api.md:868-873) - a bare type_identifier, no package_scope (compare data_type's [ class_scope | package_scope ] type_identifier, docs/systemverilog/2017/md/section-41-data-read-api.md:855); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856).
// REJECTS at the `::` on both SV profiles; the control `enum logic [7:0]` ACCEPTS.
package pkg2;
  typedef enum pkg1::uint8_t { a = 1 } e_t;
endpackage
