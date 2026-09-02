// SV-CORPUS-GRAD.13p — an OVER-ACCEPTANCE on EVERY profile, expected ACCEPT until fixed.
// IEEE 1800-2017 §28.16 (docs/systemverilog/2017/md/section-28-gate-level-and-switch-level-modeling.md:396): for the
// n-input gates "The delay specification shall be zero, one, or two delays."; Annex A A.3.1 gives n_input_gatetype
// [ drive_strength ] [ delay2 ]. IEEE 1364-2005 §7.14 (docs/verilog/2005/md/section-7-gate-and-switch-level-modeling.md:376)
// and A.3.1 (:410) say the same. Three delays belong to enable gates and switches (delay3) only.
// WHERE: the L3 class (shape / cardinality) — the shipped `delay` rule carried the triple_optional alternative at every
// gate site because the LRM extractor merged delay2/delay3 into one rule (LRM-GRAMMAR-FIDELITY.1f).
// Pinned BEFORE the fix (2026-09-02); the ratchet went RED on exactly this row on all three profiles when the delay2
// split landed (PGEN-SV-CORPUS-GRAD-0322, release 1.0.204, ledger SV-0077) and the row now guards the fix as class=invalid.
module test;
  wire a, b, o;
  nand #(1,2,3) g (o, a, b);
endmodule
