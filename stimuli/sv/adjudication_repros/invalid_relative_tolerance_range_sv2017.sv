// SV-CORPUS-GRAD.3.59 — the same text under IEEE 1800-2017, which has no tolerance ranges at all
// (A.8.3 value_range ::= expression | [ expression : expression ]): `+%-` and `+/-` both reject.
module t;
  int x, a, b;
  initial begin
    if (x inside {[a +%- b]}) x = 1;
  end
endmodule
