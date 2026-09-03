// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// interface class inte; interface class bad; endclass endclass - a nested interface class
// IEEE 1800-2017 A.1.2 interface_class_item section-41-data-read-api.md:201-206 has no class-declaration arm. t_implements_nested_bad.v l.9.
class Cls;
  interface class inte;
    interface class bad; endclass
  endclass
endclass
