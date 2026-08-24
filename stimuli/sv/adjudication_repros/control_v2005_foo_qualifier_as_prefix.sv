// SV-CORPUS-GRAD.13e.7(b) — the REJECTING isolator for accepts_invalid_v2005_class_qualifier.sv.
// Identical text with one identifier changed: `foo_qualifier` is not a PGEN keyword, so the two
// adjacent identifiers have no derivation and the parse fails. That is what proves the ACCEPT next
// door is the bogus KEYWORD and not some general two-identifier permissiveness.
// ⛔ Class `invalid`, not `control`: it is expected to REJECT forever, on every profile.
module top;
  integer y, m;
  initial y = foo_qualifier m;
endmodule
