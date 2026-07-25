// DEFECT (REJECT at baseline): SPACED `## [*]` — IEEE 1800-2017 A.2.10 alt 3
// SV is free-form: `##`, `[`, `*`, `]` are separate lexical tokens.
module m;
  logic clk, a, b;
  sequence s1; @(posedge clk) a ## [*] b; endsequence
endmodule
