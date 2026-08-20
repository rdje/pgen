// SV-CORPUS-GRAD.13c.2v — CONTROL for fixed_scope_randomize_with_constraint.sv.
// The same scope randomize with the `with constraint_block` tail removed. It parsed BEFORE the fix
// (as a plain `tf_call` — `--parse-dump-ast` reported kind "plain_tf", not "randomize") and after,
// so the flip in the fixed row is attributable to the CONSTRAINT BLOCK alone.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023.
module m;
  int a, b;
  initial begin
    if (std::randomize(a, b)) $display("ok");
    if (randomize(a)) $display("ok");
  end
endmodule
