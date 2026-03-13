`include "preprocess_deep_include_package_width_thirty_two_child_leaf.svh"

module preprocess_deep_include_package_width_triaconta_bridge_wildcard(output logic [3:0] a, output logic [3:0] b, output logic [3:0] q);
  import deep_cfg_pkg::*;
  assign a = A;
  assign b = B;
  deep_width_triaconta_bridge bridge(.*);
endmodule
