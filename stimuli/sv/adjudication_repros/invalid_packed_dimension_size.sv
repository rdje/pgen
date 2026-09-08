// SV-CORPUS-GRAD.3.48 - an INVALID pin: text no Annex A production derives - must keep REJECTING on every profile.
// reg [2] b; reg [2][3] x; - PACKED dimensions in the size form. IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (section-41-data-read-api.md:1058-1062; footnote 20 :3522 confines the unsized form to a DPI import formal); the size form [ constant_expression ] is unpacked_dimension's alone (:1046-1050); 1800-2023 A.2.5 :1085-1089; IEEE 1364-2005 A.2.5 range ::= [ msb : lsb ] (section-Annex_A-normative-formal-syntax-definition.md:310). Spelled in 1364-2005 vocabulary so it rejects for the same reason on every profile. Stands for sv2v multipack_struct_cast.sv, struct_param.sv, unbased_unsized_default.sv and iverilog display_bug.v.
module m;
  reg [2] b;
  reg [2][3] x;
endmodule
