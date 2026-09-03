// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// initial case (type(real)) type(int): ; default: ; endcase - a type reference as a PROCEDURAL case expression
// IEEE 1800-2017 A.6.7 case_expression ::= expression section-41-data-read-api.md:2239; A.8.4 primary section-41-data-read-api.md:641-656 has no type_reference (constant_primary does, section-41-data-read-api.md:3109, fn 40 section-41-data-read-api.md:3560); 6.23 section-65536-216-bits.md:1610-1631. t_type_compare_bad.v l.11; the partition question is SV-CORPUS-GRAD.3.32 - if it widens the grammar this pin flips to fixed.
module t;
  initial begin
    case (type(real))
      type(int): ;
      default: ;
    endcase
  end
endmodule
