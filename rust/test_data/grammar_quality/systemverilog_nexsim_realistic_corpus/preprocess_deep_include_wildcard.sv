`include "preprocess_deep_include_wildcard_leaf.svh"

module preprocess_deep_include_wildcard(output logic [`WILDCARD_WIDTH-1:0] q);
  logic [`WILDCARD_WIDTH-1:0] d;
  deep_wildcard_leaf u0(.*);
endmodule
