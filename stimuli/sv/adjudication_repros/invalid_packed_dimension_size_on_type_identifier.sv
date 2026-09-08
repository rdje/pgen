// SV-CORPUS-GRAD.3.51 - the PACKED size form on a typedef'd type: `S [WIDTH] s = '{...};` (sv2v test/core/struct_unit_array.sv) - the .3.48 ruling on a type_identifier: IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:855) with A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838, :1085-1089).
// REJECTS at the byte after the `[` on both SV profiles; the control `S [1:0] s;` ACCEPTS.
module m;
  typedef struct packed { logic x; } S;
  S [4] s;
endmodule
