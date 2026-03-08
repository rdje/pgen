`include "preprocess_deep_include_package_width_seven_child_leaf.svh"

module preprocess_deep_include_package_width_seven_child_pipeline(output logic [3:0] q);
  import deep_cfg_pkg::*;
  logic [3:0] s0;
  logic [3:0] s1;
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  logic [3:0] mid3;
  logic [3:0] mid4;
  logic [3:0] mid5;
  assign s0 = A;
  assign s1 = B;
  deep_width_pipe_stage u0(.a(s0), .b(s1), .y(mid0));
  deep_width_pipe_stage u1(.a(mid0), .b(s1), .y(mid1));
  deep_width_pipe_stage u2(.a(mid1), .b(s1), .y(mid2));
  deep_width_pipe_stage u3(.a(mid2), .b(s1), .y(mid3));
  deep_width_pipe_stage u4(.a(mid3), .b(s1), .y(mid4));
  deep_width_pipe_stage u5(.a(mid4), .b(s1), .y(mid5));
  deep_width_pipe_stage u6(.a(mid5), .b(s1), .y(q));
endmodule
