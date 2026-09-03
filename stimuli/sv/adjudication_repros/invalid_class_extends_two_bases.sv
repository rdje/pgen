// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// class Cls extends Base1, Base2; - two base classes
// IEEE 1800-2017 A.1.2 class_declaration: [ extends class_type [ ( list_of_arguments ) ] ] section-41-data-read-api.md:183-190. t_class_extends_bad.v l.13.
class Base1; endclass
class Base2; endclass
class Cls extends Base1, Base2;
endclass
