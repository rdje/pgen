module m;
  logic s;
  initial begin s = 1; #1 s = 0; #1; s = 1; end
endmodule
