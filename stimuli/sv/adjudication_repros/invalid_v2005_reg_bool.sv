// SV-CORPUS-GRAD.13e.13 — REJECT FOREVER on every profile.
// `reg bool [31:0] idx;` — `bool` is a CADENCE EXTENDED TYPE iverilog admits through its
// default-ON `gn_cadence_types_flag` (-gxtypes; pform.cc), never a word of either standard.
// IEEE 1364-2005 A.2.1.3 reg_declaration ::= reg [ signed ] [ range ] list_of_variable_identifiers ;
// (docs/verilog/2005/md/section-4-data-types.md:181-185) admits no type word after `reg`;
// IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842)
// derives `reg [ signing ] { packed_dimension }` and nothing a bare identifier could fill.
// Corpus rows: ivtest/ivltests/bool1.v, compare_bool_reg.v, constfunc8.v (regress-ivl1.list, normal).
// Control: control_reg_range.sv (`reg [31:0] idx;`) parses on every profile.
module t;
  reg bool [31:0] idx;
endmodule
