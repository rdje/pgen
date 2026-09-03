// SV-CORPUS-GRAD.13x.c - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// enum { A['x] } x; - an unbased unsized literal as an enum range
// IEEE 1800-2017 A.2.2.1 enum_name_declaration takes integral_number section-41-data-read-api.md:875-876; integral_number section-41-data-read-api.md:3240-3243; 'x is unbased_unsized_literal section-41-data-read-api.md:3290. sv2v test/error/enum_range_x.sv l.4.
module top;
  enum { A['x] } x;
endmodule
