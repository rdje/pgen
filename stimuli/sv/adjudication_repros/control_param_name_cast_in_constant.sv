// Control for `defect_constant_size_cast.sv`: a size cast in the SAME constant-expression position
// whose size is a NAMED parameter rather than a numeric literal. `W` is an identifier, so
// casting_type still matches on branch 1/5 (`simple_type` -> `ps_type_identifier`) — traced:
// `Rule 'casting_type' selected branch 1/5`. The cycle guard fires here too and costs nothing,
// because another casting_type alternative can match the same text.
// ⇒ the defect is not "size casts in constant expressions"; it is a size cast whose size is a
// numeric literal, for which `constant_primary` is casting_type's ONLY viable alternative.
// Expected: ACCEPT — today and after the fix.
package p;
  parameter int W = 8;
  parameter logic [7:0] K = W'(1);
endpackage
