// SV-CORPUS-GRAD.13e.13 — REJECT FOREVER on every profile.
// `reg real target;` — IEEE 1364-2005 declares a real VARIABLE with real_declaration ::=
// real list_of_real_identifiers ; (docs/verilog/2005/md/section-4-data-types.md:171-175) and
// reg_declaration ::= reg [ signed ] [ range ] list_of_variable_identifiers ; (:181-185) admits no
// `real` after `reg`; IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842)
// derives `reg` only with a signing and packed dimensions. iverilog's acceptance is its own leniency.
// Corpus row: ivtest/ivltests/real8.v (regress-ivl1.list, normal).
// Control: control_real_declaration.sv (`real target;`) parses on every profile.
module t;
  reg real target;
endmodule
