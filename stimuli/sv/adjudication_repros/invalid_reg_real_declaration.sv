// SV-CORPUS-GRAD.52 — ILLEGAL on every profile: `reg real vo;` — two data types on one declaration.
// IEEE 1364-2005 A.2.1.3 reg_declaration ::= reg [ signed ] [ range ] list_of_variable_identifiers ; and
// real_declaration ::= real list_of_real_identifiers ; (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:170-172)
// — neither derives the other's keyword; IEEE 1800-2017 A.2.2.1 data_type derives integer_vector_type (reg) OR
// non_integer_type (real), never both (docs/systemverilog/2017/md/section-41-data-read-api.md:842, :889, :898).
// iverilog 13.0 accepts it under -g2005 and -g2012 (rc 0) — a tool leniency; the clause decides. From
// ivtest/ivltests/pr1861212d.v:2. Control: control_real_declaration.sv.
module t;
  reg real vo;
endmodule
