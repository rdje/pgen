`include "preprocess_nested_include_leaf.svh"

module preprocess_nested_include_named_port(input logic d, output logic q);
  included_leaf u0(.a(d), .y(q));
endmodule
