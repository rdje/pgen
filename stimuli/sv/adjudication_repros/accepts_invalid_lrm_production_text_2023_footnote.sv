// SV-CORPUS-GRAD.13c.2m — the 1800-2023 twin of the row above. Annex A prints
// `class_qualifier := [ local ::48 ] …`, and PGEN's sv_2023 branch carries
// `kw_n_48_64e095fe := trivia /48\\b/`. ⛔ sv_2023 ONLY. Two editions, two footnote numbers, two
// literal tokens: the edition difference is what turns the extraction theory into a measurement.
// Expected today: ACCEPT on sv_2023.
module top;
  int y;
  initial y = null class_qualifier:=local::48;
endmodule
