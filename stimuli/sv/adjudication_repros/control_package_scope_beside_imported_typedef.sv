package p;
  typedef int T;
endpackage
import p::*;
package T;
  parameter int VALUE = 23;
endpackage
module test;
  initial if (T::VALUE !== 23) $stop;
endmodule
