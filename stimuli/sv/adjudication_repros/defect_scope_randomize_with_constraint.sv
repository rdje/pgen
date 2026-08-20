// SV-CORPUS-GRAD.13c.2v — DEFECT, DIAGNOSED not fixed (PGEN-SV-CORPUS-GRAD-0244).
// ⛔ Expected REJECT *today*. The fix is written, measured and BLOCKED on `.13c.2w`: it costs
// entries +1,917,021 / memo_hits +1,012,779 with committed UNCHANGED, and PARSE-COST-RATCHET has
// no coded invariant for that shape. The patch is tracked at
// docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/sv0065_fix/. When it lands this row FLIPS
// to ACCEPT and the runner will say so — a fix cannot land silently.
// IEEE 1800-2017 A.8.4: primary ::= … | function_subroutine_call | … and A.8.2 expands that to
// subroutine_call ::= tf_call | system_tf_call | method_call | [ std :: ] randomize_call.
// PGEN renders `primary`'s call alternative as `call_primary` — the postfix-chain rule the LR lift
// authored — whose alternatives do NOT include `randomize_call`, so the `with constraint_block`
// tail of a SCOPE randomize was unreachable from every expression.
//
// ⛔ THE `with` CLAUSE IS THE WHOLE POINT. `std::randomize(a, b)` on its own parsed all along —
// through `tf_call`, whose identifier matches `randomize` because IEEE 1800 Annex B does NOT
// reserve it. `tf_call` has no `with` tail, so the moment a constraint block appears the construct
// is unparseable. Its control is `control_scope_randomize_no_constraint.sv`.
// IEEE 1800-2017 §18.12 / Syntax 18-11 is the clause-body production the census named:
//   scope_randomize ::= [ std :: ] randomize ( [ variable_identifier_list ] ) [ with constraint_block ]
// Expected TODAY: REJECT on sv_2017 + sv_2023 (class=defect).
module m;
  int a, b;
  initial begin
    if (std::randomize(a, b) with { a < b; }) $display("ok");
  end
endmodule
