// SV-CORPUS-GRAD.22 (2026-09-24) — a DOMINATED arm, not a witness: this ACCEPTS on sv_2017 and
// sv_2023, and `method_call_receiver_qualifier` is entered and never commits — the package-scoped
// receiver `p::h` is read by split_direct_callable_method_call (the .3.42 split route) at every
// site measured. Re-measure with --dump-rule-outcome-counts-json: rule_committed_counts carries
// split_direct_callable_method_call=1 and no method_call_receiver_qualifier. Owned by SV-CORPUS-GRAD.33.
package p;
  class C;
    function void f(); endfunction
  endclass
  C h;
endpackage
module m;
  initial p::h.f();
endmodule
