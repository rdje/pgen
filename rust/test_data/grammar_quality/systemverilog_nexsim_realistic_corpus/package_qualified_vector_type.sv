package defs_pkg;
  parameter int W = 8;
endpackage

module m(input logic [defs_pkg::W-1:0] a, output logic [defs_pkg::W-1:0] y);
  assign y = a;
endmodule
