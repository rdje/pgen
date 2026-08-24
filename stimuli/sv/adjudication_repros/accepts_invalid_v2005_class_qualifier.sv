// SV-CORPUS-GRAD.13e.7(b) — the verilog_2005 carrier for a defect .13c.2m ALREADY OWNS.
// PGEN extracted the IEEE 1800 NONTERMINAL `class_qualifier` as a literal KEYWORD
// (`kw_class_qualifier_fa08937d := trivia /class_qualifier\b/`), so two adjacent identifiers
// parse as a primary. IEEE 1364-2005 has no such keyword and no such production at all.
// ⛔ THE REMEDY IS NOT A verilog_2005 GATE. .13c.2m deletes the bogus keyword in EVERY profile;
// gating it here would make the parser correct in one profile and leave it wrong in two.
// This row exists because .13c.2m's own carrier uses `int y, m;`, which verilog_2005 rejects for
// an unrelated reason — so the sv-profile pin could never have reported the v2005 leak.
// Control: control_v2005_foo_qualifier_as_prefix.sv (one identifier changed, and it REJECTS).
module top;
  integer y, m;
  initial y = class_qualifier m;
endmodule
