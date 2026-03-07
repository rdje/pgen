package defs_pkg;
  localparam logic [3:0] A = 4'b0101;
  localparam logic [3:0] B = 4'b1010;
endpackage

module imported_width_pair(output logic [3:0] y0, output logic [3:0] y1);
  import defs_pkg::*;
  assign y0 = A;
  assign y1 = B;
endmodule

module package_import_multi_width_bind_instantiation(output logic [3:0] q0, output logic [3:0] q1);
  logic [3:0] s0;
  logic [3:0] s1;
  imported_width_pair u0(.y0(s0), .y1(s1));
  assign q0 = s0;
  assign q1 = s1;
endmodule
