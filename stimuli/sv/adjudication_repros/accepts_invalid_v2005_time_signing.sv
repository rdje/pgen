// SV-CORPUS-GRAD.13e.4 — the SECOND atom the same over-acceptance reaches, and a DIFFERENT LRM
// production, so it is its own row rather than a restatement of the integer one.
// IEEE 1364-2005 A.2.1.3 is
//   time_declaration ::= time list_of_variable_identifiers ;
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:174) — no signing.
// ⭐ `integer_atom_type` admits exactly `integer` and `time` under verilog_2005, so a fix that
// gated only the `integer` spelling would leave this one live; this row is what proves it did not.
// FIXED by SV-CORPUS-GRAD.13e.4: REJECT FOREVER on verilog_2005, ACCEPT on sv_2017/sv_2023.
module test;
  time signed t;
  initial t = 0;
endmodule
