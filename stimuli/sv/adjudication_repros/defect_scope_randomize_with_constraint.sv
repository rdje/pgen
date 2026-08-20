// SV-CORPUS-GRAD.13c.2v — FIXED 2026-08-20 (PGEN-SV-CORPUS-GRAD-0256); diagnosed by -0244.
// ⭐ This row was filed as `class=defect` expecting REJECT, so the runner FAILED with
// "flip it to ACCEPT" on the commit that landed the fix — a fix cannot land silently here.
// It now guards the fix against regression forever, and its `arm` claim (`scope_randomize`)
// pins WHICH alternative parses it: exactly one rule in the grammar emits that kind.
// ⛔ THE COST WAS REAL AND WAS PAID THROUGH THE RATCHET, NOT AROUND IT. The fix costs entries
// +1,917,021 / memo_hits +1,012,779 with committed UNCHANGED, which fits none of
// PARSE-COST-RATCHET's three arithmetic identities. It landed under the FOURTH invariant,
// `contained_in_introduced_subgraph` (SV-CORPUS-GRAD.13c.2w), which asks a per-rule question
// three totals cannot: is every rule whose entries ROSE reachable, in the grammar's own
// reference graph, from the rule the change INTRODUCED?
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
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023, through a `scope_randomize` arm (class=fixed).
module m;
  int a, b;
  initial begin
    if (std::randomize(a, b) with { a < b; }) $display("ok");
  end
endmodule
