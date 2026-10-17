// SV-CORPUS-GRAD.60 — the SAME text under IEEE 1800-2023, where A.2.4 param_assignment takes `{ variable_dimension }`
// (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1042) and unsized_dimension ::= [ ] (:1106).
// The 2023 side of invalid_param_unsized_dimension_2017.sv. Expected: ACCEPT.
package p;
  parameter string L[] = {"a", "b"};
endpackage
