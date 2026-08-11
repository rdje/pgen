// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. `~&` is a UNARY reduction operator only; the
// binary bitwise set is `& | ^ ^~ ~^` (IEEE 1800-2017 Table 11-3 / A.8.6).
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module m;
  logic [3:0] a, b, c;
  initial c = a ~& b;
endmodule
