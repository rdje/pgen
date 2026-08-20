// SV-CORPUS-GRAD.13c.2v — FIXED 2026-08-20 (PGEN-SV-CORPUS-GRAD-0256); diagnosed by -0244.
// ⭐ This row was filed as `class=defect` expecting REJECT, so the runner FAILED with
// "flip it to ACCEPT" on the commit that landed the fix — a fix cannot land silently here.
// It now guards the fix against regression forever, and its `arm` claim (`scope_randomize`)
// pins WHICH alternative parses it: exactly one rule in the grammar emits that kind.
// The three host contexts a scope randomize actually appears in, in real constrained-random code:
//   * negated inside an `if` — `if (!std::randomize(a, b) with { … })`, the canonical failure check
//   * discarded through a void cast — `void'(std::randomize(c) with { … })`
//   * as an immediate assertion's expression — `assert (randomize(a) with { … })`
// All three are ordinary EXPRESSION positions, which is exactly what SV-0065 made unreachable: they
// reach `primary`, and `primary`'s call alternative is `call_primary`, which has no `randomize_call`.
// ⛔ Kept as its own row rather than folded into defect_scope_randomize_with_constraint.sv because
// the void-cast context routes through `void_cast_statement_sv_only`, which references
// `function_subroutine_call` DIRECTLY — one of the three sites that always carried the alternative —
// so this row also pins that the two routes agree.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023 (class=fixed). ⚠️ THREE randomize calls but only
// TWO `scope_randomize` nodes, and that asymmetry is the row's point: the void-cast context
// reaches `function_subroutine_call` DIRECTLY, one of the three sites that always carried the
// alternative, so it never needed the new rule. The two routes agree on the verdict and differ
// in shape, which is exactly what this row exists to pin.
module m;
  int a, b, c;
  task t;
    if (!std::randomize(a, b) with { a < b; a > 0; }) $error("randomization failed");
    void'(std::randomize(c) with { c inside {[1:10]}; });
    assert (randomize(a) with { a != 0; }) else $error("no");
  endtask
endmodule
