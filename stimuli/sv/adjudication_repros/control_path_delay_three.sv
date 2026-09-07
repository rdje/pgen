module m(input a, output b);
  specify
    (a => b) = (1, 2, 3);
  endspecify
endmodule
