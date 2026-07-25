// CONTROL (must ACCEPT at baseline): literal token `##[+]` — alt 4
module m;
  logic clk, a, b;
  sequence s1; @(posedge clk) a ##[+] b; endsequence
endmodule
