// SV-CORPUS-GRAD.3.46 - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// nettype wT x with f; - the §6.6.7 ALIAS form (base = a declared nettype) carrying a resolution function: A.2.1.3's second production `nettype [ scope ] net_type_identifier net_type_identifier ;` has no `with`, and its first production takes a data_type, which a nettype identifier is not.
// IEEE 1800-2017 A.2.1.3 net_type_declaration section-41-data-read-api.md:828-833; 6.6.7 section-6-data-types.md:617-619 (the resolution function belongs to the data-type form) and :692 (the second form is another name for an existing nettype). Through 1.0.219 this parsed down the data-type alternative: the alias-head guard required a `;` after the head, and `with` is not `;`.
typedef struct { int a; } T;
function T f(input T a[]); return a[0]; endfunction
nettype T wT with f;
nettype wT x with f;
module m; endmodule
