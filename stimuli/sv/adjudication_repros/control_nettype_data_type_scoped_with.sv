// SV-CORPUS-GRAD.3.46 - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// nettype p::T x with p::f; - A.2.1.3's FIRST production with a package-scoped TYPEDEF as its data_type and a scoped resolution function: the alias-head guard's `checked_nettype_identifier` refuses `T` (a typedef is not a declared nettype), so the data-type alternative proceeds exactly as before.
// IEEE 1800-2017 A.2.1.3 net_type_declaration section-41-data-read-api.md:828-833; 6.6.7 section-6-data-types.md:617-619.
package p;
  typedef struct { int a; } T;
  function T f(input T a[]); return a[0]; endfunction
endpackage
module m;
  nettype p::T x with p::f;
endmodule
