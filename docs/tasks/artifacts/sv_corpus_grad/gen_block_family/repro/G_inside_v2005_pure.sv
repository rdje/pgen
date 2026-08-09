module m;
  reg a;
  reg b;
  initial b = a inside {1'b0, 1'b1};
endmodule
