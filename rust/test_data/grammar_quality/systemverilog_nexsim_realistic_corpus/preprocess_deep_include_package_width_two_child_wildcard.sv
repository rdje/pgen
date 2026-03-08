`include "preprocess_deep_include_package_width_two_child_leaf.svh"

module preprocess_deep_include_package_width_two_child_wildcard(output logic [3:0] y0, output logic [3:0] y1);
  import deep_cfg_pkg::*;
  logic [3:0] a;
  logic [3:0] b;
  assign a = A;
  assign b = B;
  deep_width_pipe_stage u0(.*);
  deep_width_pipe_stage u1(.*);
endmodule
