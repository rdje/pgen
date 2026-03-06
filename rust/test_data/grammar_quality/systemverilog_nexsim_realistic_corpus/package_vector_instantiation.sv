package defs_pkg;
  parameter int W = 4;
endpackage

module child(input logic [defs_pkg::W-1:0] a, output logic [defs_pkg::W-1:0] y);
  assign y = a;
endmodule

module top(input logic [defs_pkg::W-1:0] d, output logic [defs_pkg::W-1:0] q);
  child u0(.a(d), .y(q));
endmodule
