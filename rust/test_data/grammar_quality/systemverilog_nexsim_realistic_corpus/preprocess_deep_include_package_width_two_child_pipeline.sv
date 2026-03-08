`include "preprocess_deep_include_package_width_two_child_leaf.svh"

module preprocess_deep_include_package_width_two_child_pipeline(output logic [3:0] q);
  import deep_cfg_pkg::*;
  logic [3:0] s0;
  logic [3:0] s1;
  logic [3:0] mid;
  assign s0 = A;
  assign s1 = B;
  deep_width_pipe_stage u0(.a(s0), .b(s1), .y(mid));
  deep_width_pipe_stage u1(.a(mid), .b(s1), .y(q));
endmodule
