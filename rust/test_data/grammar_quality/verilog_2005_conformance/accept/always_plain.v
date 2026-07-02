module m;
  reg q;
  always @(q) q = ~q;
endmodule
