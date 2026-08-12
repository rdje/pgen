// ENGINE-UNIVERSAL-SERVICES.13 — probe for SV cycles 1 and 5 (the cast/call knot).
//
// Cycle (12 rules, reported by 9 of them):
//   call_primary -> call_with_postfix_chain -> chainable_call_initial ->
//   direct_callable_method_call -> method_call_root -> method_call_receiver ->
//   method_call_receiver_sv_2017 -> cast -> casting_type -> constant_primary ->
//   constant_primary_sv_2017 -> constant_function_call -> call_primary
//
// The derivation that NEEDS the recursion: a size cast whose casting_type is a
// constant_function_call. IEEE 1800-2017 A.8.4 makes it derivable text —
//   casting_type ::= … | constant_primary
//   constant_primary ::= … | constant_function_call
// — and §6.24.1 requires only that the size be a constant expression evaluating to a
// positive integer, which a constant function call is.
//
// Expected today: REJECT under sv_2017 AND sv_2023 (furthest_position at the `'`).
// Expected once .13 lands: ACCEPT under both.
module m;
  logic [7:0] k;
  function automatic int w(); return 8; endfunction
  initial k = w()'(1);
endmodule
