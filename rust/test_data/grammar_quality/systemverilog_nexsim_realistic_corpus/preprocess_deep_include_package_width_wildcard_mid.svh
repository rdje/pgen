`include "preprocess_deep_include_package_width_wildcard_defs.svh"

module deep_width_wildcard(output logic [3:0] a, output logic [3:0] b);
  import deep_width_pkg::*;
  assign a = A;
  assign b = B;
endmodule
