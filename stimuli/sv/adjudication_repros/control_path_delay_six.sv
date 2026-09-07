module m(input a, output b);
  specify
    (a => b) = (1, 2, 3, 4, 5, 6);
  endspecify
endmodule
