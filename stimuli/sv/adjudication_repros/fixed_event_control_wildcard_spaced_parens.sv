// SV-CORPUS-GRAD.3.58 — the implicit event expression with white space INSIDE the parentheses.
// IEEE 1800-2017 A.6.5 `event_control ::= … | @ (*)` in a notation that separates tokens by white space,
// spelled out by IEEE 1800-2023 A.6.5 as `@ ( * )`; IEEE 1364-2005 A.6.5 `@ (*)` likewise.
// Through release 1.0.227 the 2017/2005 rule carried only `@*` and the glued `(*)` (an attr_open + `)`),
// so a space after the `(` — `@( * )`, `@ ( * )`, `@ ( *)` — rejected (sv2v test/basic/sense_star.sv,
// OpenTitan hw/top_*/ip/ast/rtl/*_pgd.sv:29 `always @( * ) begin`).
module t;
  reg x, y;
  always @( * ) y = x;
  always @ ( * ) y = x;
  always @ ( *) y = x;
  always @( * ) begin y = x; end
endmodule
