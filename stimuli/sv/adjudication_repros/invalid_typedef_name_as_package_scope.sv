typedef int T;
module test;
  initial if (T::VALUE !== 23) $stop;
endmodule
