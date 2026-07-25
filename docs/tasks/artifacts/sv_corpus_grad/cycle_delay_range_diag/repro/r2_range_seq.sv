// LRM 1800-2017 §9 (:896): `@(negedge clk) d ##[2:5] e;`
module m;
  logic clk, d, e;
  sequence s1; @(negedge clk) d ##[2:5] e; endsequence
endmodule
