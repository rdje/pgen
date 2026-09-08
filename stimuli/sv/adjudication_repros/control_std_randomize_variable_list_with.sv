module m;
  int a, b;
  initial if (std::randomize(a, b) with { 2 < a; a < 7; b < a; } != 1) $stop;
endmodule
