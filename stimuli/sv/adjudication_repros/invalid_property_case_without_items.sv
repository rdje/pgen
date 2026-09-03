// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// property p2; case (sel) endcase endproperty - a property case with no item
// IEEE 1800-2017 A.2.10 case ( expression_or_dist ) property_case_item { property_case_item } endcase section-41-data-read-api.md:1337. t_property_case_bad.v l.18.
module t; logic [1:0] sel;
  property p2; case (sel) endcase endproperty
endmodule
