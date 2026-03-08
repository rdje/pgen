package defs_pkg;
  localparam logic [3:0] A = 4'b0110;
  localparam logic [3:0] B = 4'b1001;
endpackage

module imported_width_pair(output logic [3:0] a, output logic [3:0] b);
  import defs_pkg::*;
  assign a = A;
  assign b = B;
endmodule

module width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module package_import_multi_width_five_child_pipeline(output logic [3:0] q);
  logic [3:0] s0;
  logic [3:0] s1;
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  logic [3:0] mid3;
  imported_width_pair src(.a(s0), .b(s1));
  width_pipe_stage u0(.a(s0), .b(s1), .y(mid0));
  width_pipe_stage u1(.a(mid0), .b(s1), .y(mid1));
  width_pipe_stage u2(.a(mid1), .b(s1), .y(mid2));
  width_pipe_stage u3(.a(mid2), .b(s1), .y(mid3));
  width_pipe_stage u4(.a(mid3), .b(s1), .y(q));
endmodule
