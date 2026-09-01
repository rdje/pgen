// SV-CORPUS-GRAD.13o — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.7.4 edge_identifier (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:883) ::= posedge | negedge — no `edge`;
// A.6.5 event_expression (:713-717) admits `posedge expression` and `negedge expression` only. `edge` in an event expression is IEEE 1800 (A.7.4 / A.6.5).
// WHERE: the L2 class — `edge` IS a 1364-2005 Annex B keyword (timing-check edge_control_specifier, A.7.5.3), admitted in a POSITION the LRM never allows;
// invisible to the keyword-level census (TOOLBOX 5.11), found by the keyword-enumeration membership census (TOOLBOX 5.13) `extra` column.
// Three legs measured 2026-09-01: rc 0 under verilog_2005 (leg 1), rc 0 under sv_2017 (leg 2), `always @(zzq c)` rc 1 under verilog_2005 (leg 3).
// Pinned BEFORE the fix (2026-09-01); the ratchet went RED on exactly this row when the gate landed
// (PGEN-SV-CORPUS-GRAD-0319, release 1.0.203, ledger SV-0076) and the row now guards the fix as class=invalid.
module test;
  reg c;
  always @(edge c) ;
endmodule
