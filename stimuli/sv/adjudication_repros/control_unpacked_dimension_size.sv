// SV-CORPUS-GRAD.3.48 - a CONTROL: the size form where the standard admits it, the UNPACKED dimension - must keep ACCEPTING under the SV profiles.
// logic x [2]; logic y [2][3]; int q [4]; - IEEE 1800-2017 A.2.5 unpacked_dimension ::= [ constant_range ] | [ constant_expression ] (section-41-data-read-api.md:1046-1050); the same text is illegal under IEEE 1364-2005, whose dimension is a range (accepts_invalid_v2005_unpacked_dimension_size.sv).
module m;
  logic x [2];
  logic y [2][3];
  int q [4];
endmodule
