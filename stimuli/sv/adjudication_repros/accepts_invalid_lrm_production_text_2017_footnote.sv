// SV-CORPUS-GRAD.13c.2m — ⭐ THE SMOKING GUN: the LRM's OWN TEXT for a production, parsed as a
// SystemVerilog expression, footnote superscript and all. IEEE 1800-2017 A.8.4 prints
// `class_qualifier := [ local ::43 ] …` where `43` is a FOOTNOTE MARKER, and PGEN carries
// `kw_n_43_0286dd55 := trivia /43\\b/` as a literal token on the sv_2017 branch.
// ⛔ sv_2017 ONLY — 1800-2023 renumbers that footnote to 48, which is why the sibling file exists
// and why the pair proves the mechanism twice. Expected today: ACCEPT on sv_2017.
module top;
  int y;
  initial y = null class_qualifier:=local::43;
endmodule
