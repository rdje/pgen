// SV-CORPUS-GRAD.3.52 - a PARENTHESISED type in a variable declaration: `(states_t) state1;` (Surelog tests/EnumParenType/dut.sv:9 - the file's own comment: "Test parenthesized type variable declaration: (type) varname;"). IEEE 1800-2017 A.2.1.3 data_declaration ::= [ const ] [ var ] [ lifetime ] data_type_or_implicit list_of_variable_decl_assignments ; | … (docs/systemverilog/2017/md/section-41-data-read-api.md:779) with A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - no alternative parenthesises a type; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:756, :825-849).
// REJECTS at the `(` on both SV profiles (the furthest byte is the end of the preceding declaration); the control `states_t state1;` ACCEPTS.
module m;
  typedef enum {A, B} states_t;
  (states_t) state1;
endmodule
