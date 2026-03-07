package defs_pkg;
  localparam int A = 1;
  localparam int B = 2;
endpackage

module imported_child(output int y0, output int y1);
  import defs_pkg::*;
  assign y0 = A;
  assign y1 = B;
endmodule

module package_import_multi_bind_instantiation;
  int s0;
  int s1;
  imported_child u0(.y0(s0), .y1(s1));
endmodule
