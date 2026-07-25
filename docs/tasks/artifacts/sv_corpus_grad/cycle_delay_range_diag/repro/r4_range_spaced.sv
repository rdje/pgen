// ispras ieee-1800-2012/16/16.09.08_01.sv: `te1 ## [2:5] te2;`
module m;
  logic clk, te1, te2;
  sequence s1; @(posedge clk) te1 ## [2:5] te2; endsequence
endmodule
