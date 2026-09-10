// SV-CORPUS-GRAD.3.59 — the relative tolerance range `[A +%- B]` of IEEE 1800-2023 11.4.13
// (A.8.3 value_range ::= … | [ expression +%- expression ]; A.6.7.1 covergroup_value_range likewise).
// Through 1.0.228 the sv_2023 profile derived `+/-` and refused `+%-` (verilator t_inside_tolerance_unsup.v:20).
module t;
  int x, a, b;
  covergroup cg;
    coverpoint x { bins lo = {[10 +%- 25]}; bins hi = {[100 +/- 5]}; }
  endgroup
  initial begin
    if (x inside {[a +%- b]}) x = 1;
    if (x inside {[a +/- b]}) x = 2;
  end
endmodule
