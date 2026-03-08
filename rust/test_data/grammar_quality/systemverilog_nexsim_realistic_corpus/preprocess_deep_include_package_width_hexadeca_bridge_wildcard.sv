`include "preprocess_deep_include_package_width_eighteen_child_leaf.svh"

module preprocess_deep_include_package_width_hexadeca_bridge_wildcard(output logic [3:0] a, output logic [3:0] b, output logic [3:0] q);
  import deep_cfg_pkg::*;
  assign a = A;
  assign b = B;
  deep_width_hexadeca_bridge bridge(.*);
endmodule
