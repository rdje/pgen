// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// initial case (type(real)) type(int): ; default: ; endcase - a type reference as a PROCEDURAL case expression
// IEEE 1800-2017 A.6.7 case_expression ::= expression section-41-data-read-api.md:2239; A.8.4 primary section-41-data-read-api.md:3122-3140 has no type_reference (constant_primary does, section-41-data-read-api.md:3109, fn 40 section-41-data-read-api.md:3560); 6.23 section-65536-216-bits.md:1610-1631. t_type_compare_bad.v l.11; RULED by SV-CORPUS-GRAD.3.32 (2026-09-07): the letter stands in both editions - a STANDING invalid pin (the primary citation corrected from :641-656, the constraint_primary block).
module t;
  initial begin
    case (type(real))
      type(int): ;
      default: ;
    endcase
  end
endmodule
