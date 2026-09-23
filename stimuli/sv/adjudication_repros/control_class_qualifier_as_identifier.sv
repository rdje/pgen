// SV-CORPUS-GRAD.13c.2m - an IDENTITY CONTROL - ACCEPT before and after release 1.0.244, the typed AST byte-identical (both SV profiles).
// `class_qualifier` used as an ordinary IDENTIFIER: it is an Annex A NONTERMINAL name, not an Annex B keyword, so a variable may carry it.
// Proves the extracted literal token never stole the identifier (measured on 1.0.243: ACCEPT) and its deletion changes nothing here.
// IEEE 1800-2017 Annex B (no such keyword); 5.6 identifiers
module m;
  int class_qualifier;
  initial class_qualifier = 1;
endmodule
