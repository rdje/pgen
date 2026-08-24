// SV-CORPUS-GRAD.13e.4 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// IEEE 1364-2005 A.2.1.3 is
//   integer_declaration ::= integer list_of_variable_identifiers ;
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:152) — there is
// no signing on it, and neither `signing` nor `integer_atom_type` appears anywhere in that Annex.
// Signing on an integer ATOM is IEEE 1800 A.2.2.1 only, so `integer unsigned u;` has no
// IEEE 1364-2005 derivation.
// ⛔ IEEE 1800 keeps `integer_atom_type [ signing ]`, so this is a verilog_2005-ONLY defect and the
// fix must be profile-gated (the `scope_randomize_sv_only` / `hierarchical_root_prefix_sv_only`
// idiom), not an inline tightening — `integer_atom_type` itself must STAY reachable under
// verilog_2005, which is what makes plain `integer u;` parse. Control: control_v2005_integer_no_signing.sv.
// FIXED by SV-CORPUS-GRAD.13e.4: REJECT FOREVER on verilog_2005, ACCEPT on sv_2017/sv_2023.
module test;
  integer unsigned u;
  initial u = 0;
endmodule
