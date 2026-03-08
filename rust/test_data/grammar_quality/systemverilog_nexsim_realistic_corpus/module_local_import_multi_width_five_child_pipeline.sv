package cfg_pkg;
  localparam logic [3:0] A = 4'b0001;
  localparam logic [3:0] B = 4'b0010;
endpackage

module width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module module_local_import_multi_width_five_child_pipeline(output logic [3:0] q);
  import cfg_pkg::*;
  logic [3:0] s0;
  logic [3:0] s1;
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  logic [3:0] mid3;
  assign s0 = A;
  assign s1 = B;
  width_pipe_stage u0(.a(s0), .b(s1), .y(mid0));
  width_pipe_stage u1(.a(mid0), .b(s1), .y(mid1));
  width_pipe_stage u2(.a(mid1), .b(s1), .y(mid2));
  width_pipe_stage u3(.a(mid2), .b(s1), .y(mid3));
  width_pipe_stage u4(.a(mid3), .b(s1), .y(q));
endmodule
