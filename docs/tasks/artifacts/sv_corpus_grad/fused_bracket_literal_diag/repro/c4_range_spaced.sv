// CONTROL (ACCEPT since .3.8, must HOLD): spaced `## [1:2]` — alt 2 admits BOTH spacings
module m;
  logic clk, a, b;
  sequence s1; @(posedge clk) a ## [1:2] b; endsequence
endmodule
