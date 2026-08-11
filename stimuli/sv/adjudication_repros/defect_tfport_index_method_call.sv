// SV-CORPUS-GRAD.13c.2 — CONFIRMED DEFECT (rejects-valid).
// `ral.arr[0].g()` is rejected when the receiver `ral` is a SUBROUTINE FORMAL ARGUMENT.
// IEEE 1800-2017 A.8.2: method_call ::= method_call_root . method_call_body,
//                       method_call_root ::= primary | implicit_class_handle
// Diagnosed mechanism (TOOLBOX §2.2/§2.4): `method_call_receiver_sv_2017` selects
// branch 2/14 under branch_policy=longest_match — the un-fact-gated
// `hierarchical_sequence_identifier` arm — which swallows `ral.arr[0].g`, leaving `()`
// with no `. method_identifier` to bind to.
// Expected today: REJECT.  Expected once fixed: ACCEPT.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w;
  function int f(rb ral); return ral.arr[0].g(); endfunction
endclass
