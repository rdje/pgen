// SV-CORPUS-GRAD.13c.2v — DEFECT, DIAGNOSED not fixed (PGEN-SV-CORPUS-GRAD-0244).
// Expected REJECT today; flips to ACCEPT when the `.13c.2w`-blocked fix lands.
// The `std::`-less spelling of the same construct. IEEE 1800-2017 §18.12 makes the package
// qualifier optional (`[ std :: ]`), and A.2.10's randomize_call also admits the parenthesised
// identifier list after `with`. Both forms went through `call_primary`, so both were unreachable.
// Expected TODAY: REJECT on sv_2017 + sv_2023 (class=defect).
module m;
  int a, b, v;
  initial begin
    v = randomize(a, b) with { a < b; };
    if (randomize(a) with (a) { a > 0; }) $display("ok");
  end
endmodule
