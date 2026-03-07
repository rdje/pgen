`include "preprocess_deep_include_package_width_defs.svh"

module deep_width_child(output logic [3:0] y);
  import deep_cfg_pkg::*;
  assign y = WIDTH_VALUE;
endmodule
