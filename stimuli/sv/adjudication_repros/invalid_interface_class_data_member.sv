// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// interface class Icls; int badi; endclass - a data member in an interface class
// IEEE 1800-2017 A.1.2 interface_class_item section-41-data-read-api.md:201-206 (type_declaration | interface_class_method | local_parameter_declaration ; | parameter_declaration ; | ;). t_implements_contents_bad.v l.8.
interface class Icls;
  int badi;
endclass
