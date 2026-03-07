package p1;
  localparam int A = 1;
endpackage

package p2;
  localparam int item = 2;
endpackage

import p1::*;
import p2::item;

module multi_imported_packages;
  int a;
  int b;
  assign a = p1::A;
  assign b = p2::item;
endmodule
