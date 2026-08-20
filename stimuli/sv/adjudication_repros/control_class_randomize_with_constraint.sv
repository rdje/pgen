// SV-CORPUS-GRAD.13c.2v — CONTROL for fixed_scope_randomize_with_constraint.sv.
// The CLASS-METHOD randomize-with (IEEE 1800-2023 Syntax 18-10 `inline_constraint_declaration`,
// which that edition also marks `// not in Annex A`). It reaches `randomize_call` through
// `method_call`, an entirely different route, so it parsed before the fix and after — which is why
// the defect survived: the overwhelmingly common UVM spelling was never affected.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023.
class C;
  rand int a;
endclass
module m;
  C c;
  initial if (c.randomize() with { a < 5; }) $display("ok");
endmodule
