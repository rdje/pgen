// SV-CORPUS-GRAD.13c.2v — DEFECT, DIAGNOSED not fixed (PGEN-SV-CORPUS-GRAD-0244).
// Expected REJECT today; flips to ACCEPT when the `.13c.2w`-blocked fix lands.
// The three host contexts a scope randomize actually appears in, in real constrained-random code:
//   * negated inside an `if` — `if (!std::randomize(a, b) with { … })`, the canonical failure check
//   * discarded through a void cast — `void'(std::randomize(c) with { … })`
//   * as an immediate assertion's expression — `assert (randomize(a) with { … })`
// All three are ordinary EXPRESSION positions, which is exactly what SV-0065 made unreachable: they
// reach `primary`, and `primary`'s call alternative is `call_primary`, which has no `randomize_call`.
// ⛔ Kept as its own row rather than folded into fixed_scope_randomize_with_constraint.sv because
// the void-cast context routes through `void_cast_statement_sv_only`, which references
// `function_subroutine_call` DIRECTLY — one of the three sites that always carried the alternative —
// so this row also pins that the two routes agree.
// Expected TODAY: REJECT on sv_2017 + sv_2023 (class=defect).
module m;
  int a, b, c;
  task t;
    if (!std::randomize(a, b) with { a < b; a > 0; }) $error("randomization failed");
    void'(std::randomize(c) with { c inside {[1:10]}; });
    assert (randomize(a) with { a != 0; }) else $error("no");
  endtask
endmodule
