`include "preprocess_deep_include_leaf.svh"

module preprocess_deep_include_named_port(input logic d0, input logic d1, output logic q);
  deep_leaf u0(.a(d0), .b(d1), .y(q));
endmodule
