// CONTROL (ACCEPT at baseline, must HOLD): tight `##[*]` — alt 3
module m;
  logic clk, a, b;
  sequence s1; @(posedge clk) a ##[*] b; endsequence
endmodule
