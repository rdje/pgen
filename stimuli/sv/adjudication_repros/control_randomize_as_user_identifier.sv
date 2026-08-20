// SV-CORPUS-GRAD.13c.2v — CONTROL for the SV-0065 fix, guarding the direction the fix could break.
// IEEE 1800 Annex B does NOT reserve `randomize` (it lists `rand`, `randc`, `randcase`,
// `randsequence` and stops), so `randomize` is a perfectly legal user identifier: a variable name,
// a user-defined function name, and a package-scoped function name. Adding a `randomize_call`
// alternative to `primary` is exactly the kind of change that could capture those, so this row
// pins that it did not.
// The `!scope_randomize` arm claim is the load-bearing half: a verdict alone cannot distinguish
// "still parses" from "still parses, but now through the new alternative".
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023.
package p;
  function int randomize(int x);
    return x;
  endfunction
endpackage
module m;
  int randomize;
  int y;
  function int local_randomize(int x);
    return x + 1;
  endfunction
  initial begin
    randomize = 1;
    y = local_randomize(3);
    y = p::randomize(4);
    $display(randomize + y);
  end
endmodule
