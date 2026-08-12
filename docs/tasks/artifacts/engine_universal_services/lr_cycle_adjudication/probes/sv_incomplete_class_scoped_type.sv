// ENGINE-UNIVERSAL-SERVICES.13 — probe for SV cycle 4 (incomplete_class_scoped_type, sv_2023 only).
//
// Cycle (2 rules, reported by both):
//   incomplete_class_scoped_type -> incomplete_class_scoped_type_sv_2023 ->
//   incomplete_class_scoped_type
//
// IEEE 1800-2023 A.2.2.1:
//   incomplete_class_scoped_type ::= type_identifier :: type_identifier_or_class_type
//                                  | incomplete_class_scoped_type :: type_identifier_or_class_type
// so a THREE-or-more-segment scoped name needs the recursive alternative.
//
// ⚠️ This probe ACCEPTS — and that is NOT proof the cycle is harmless. Traced, the guard fires
// (`Infinite recursion detected in rule 'incomplete_class_scoped_type'`), the recursive
// alternative loses, and the text is served by the SIBLING branch:
// `data_type_or_incomplete_class_scoped_type_sv_2023 selected branch 1/2` = `data_type`, whose own
// scoped-type path accepts the same token shape. ⇒ verdict DEAD-BUT-COVERED: the alternative is
// unreachable, and no LRM-grounded input has been found that ONLY it can derive.
//
// Expected today: ACCEPT under sv_2023 (and under sv_2017, via data_type alone).
// Expected once .13 lands: still ACCEPT — but then via the declared derivation.
typedef A::B::C::D t4;
