`include "preprocess_deep_include_package_width_three_child_defs.svh"

module deep_width_wildcard_bridge(input logic [3:0] a, input logic [3:0] b, output logic [3:0] q);
  logic [3:0] mid0;
  logic [3:0] mid1;
  deep_width_pipe_stage u0(.a(a), .b(b), .y(mid0));
  deep_width_pipe_stage u1(.a(mid0), .b(b), .y(mid1));
  deep_width_pipe_stage u2(.a(mid1), .b(b), .y(q));
endmodule
