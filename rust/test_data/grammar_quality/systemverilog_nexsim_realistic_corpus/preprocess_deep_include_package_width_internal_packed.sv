`include "preprocess_deep_include_package_width_internal_packed_leaf.svh"

module preprocess_deep_include_package_width_internal_packed(output logic [3:0] q);
  import deep_cfg_pkg::*;
  logic [3:0] d;
  assign d = WIDTH_VALUE;
  deep_pkg_width_child u0(.a(d), .y(q));
endmodule
