package defs_pkg;
  localparam logic [3:0] A = 4'b0101;
  localparam logic [3:0] B = 4'b1010;
endpackage

module imported_width_wildcard(output logic [3:0] a, output logic [3:0] b);
  import defs_pkg::*;
  assign a = A;
  assign b = B;
endmodule

module package_import_multi_width_wildcard_instantiation(output logic [3:0] a, output logic [3:0] b);
  imported_width_wildcard u0(.*);
endmodule
