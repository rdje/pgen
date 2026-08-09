module m;
  localparam int A = 1;
  localparam int B = A inside {1, 2};
endmodule
