// SV-CORPUS-GRAD.3.46 - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// nettype p::wT x with p::f; - the scoped ALIAS form (A.2.1.3 `nettype [ package_scope | class_scope ] net_type_identifier net_type_identifier ;`) carrying a resolution function it has no slot for.
// IEEE 1800-2017 A.2.1.3 net_type_declaration section-41-data-read-api.md:828-833; 6.6.7 section-6-data-types.md:617-619 and :692. The one-token-legal twin is control_nettype_data_type_scoped_with.sv (a scoped TYPEDEF base), which must keep ACCEPTING.
package p;
  typedef struct { int a; } T;
  function T f(input T a[]); return a[0]; endfunction
  nettype T wT with f;
endpackage
module m;
  nettype p::wT x with p::f;
endmodule
