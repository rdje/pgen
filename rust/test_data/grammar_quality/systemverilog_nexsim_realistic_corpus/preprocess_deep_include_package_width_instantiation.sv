`include "preprocess_deep_include_package_width_leaf.svh"

module preprocess_deep_include_package_width_instantiation(output logic [3:0] q);
  logic [3:0] d;
  deep_width_child u0(.y(d));
  assign q = d;
endmodule
