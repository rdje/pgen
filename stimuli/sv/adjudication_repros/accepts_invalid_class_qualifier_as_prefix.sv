// SV-CORPUS-GRAD.13c.2m — a KNOWN OVER-ACCEPTANCE (class `accepts_invalid`), pinned so it is
// WATCHED rather than merely written down. IEEE 1800 Annex B has no keyword `class_qualifier`;
// A.8.4 has a NONTERMINAL of that name. PGEN extracted it as a literal keyword
// (`kw_class_qualifier_fa08937d := trivia /class_qualifier\\b/`), so this text — two adjacent
// identifiers, which no Annex A production derives — parses as a primary.
// Isolated by `invalid_foo_qualifier_as_prefix.sv`: one identifier changed, and it REJECTS.
// Expected today: ACCEPT. When .13c.2m lands the runner FAILS with "flip it to invalid".
module top;
  int y, m;
  initial y = class_qualifier m;
endmodule
