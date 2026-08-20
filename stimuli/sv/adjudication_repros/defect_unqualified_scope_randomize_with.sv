// SV-CORPUS-GRAD.13c.2v — FIXED 2026-08-20 (PGEN-SV-CORPUS-GRAD-0256); diagnosed by -0244.
// ⭐ This row was filed as `class=defect` expecting REJECT, so the runner FAILED with
// "flip it to ACCEPT" on the commit that landed the fix — a fix cannot land silently here.
// It now guards the fix against regression forever, and its `arm` claim (`scope_randomize`)
// pins WHICH alternative parses it: exactly one rule in the grammar emits that kind.
// The `std::`-less spelling of the same construct. IEEE 1800-2017 §18.12 makes the package
// qualifier optional (`[ std :: ]`), and A.2.10's randomize_call also admits the parenthesised
// identifier list after `with`. Both forms went through `call_primary`, so both were unreachable.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023 (class=fixed). BOTH spellings must route through
// the new rule, which is why the arm claim is checked against TWO `scope_randomize` nodes.
module m;
  int a, b, v;
  initial begin
    v = randomize(a, b) with { a < b; };
    if (randomize(a) with (a) { a > 0; }) $display("ok");
  end
endmodule
