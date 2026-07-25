// DEFECT (REJECT at baseline): SPACED `## [+]` — IEEE 1800-2017 A.2.10 alt 4
module m;
  logic clk, a, b;
  sequence s1; @(posedge clk) a ## [+] b; endsequence
endmodule
