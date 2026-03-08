`include "preprocess_deep_include_package_width_fourteen_child_leaf.svh"

module preprocess_deep_include_package_width_dodeca_bridge_wildcard(output logic [3:0] a, output logic [3:0] b, output logic [3:0] q);
  import deep_cfg_pkg::*;
  assign a = A;
  assign b = B;
  deep_width_dodeca_bridge bridge(.*);
endmodule
