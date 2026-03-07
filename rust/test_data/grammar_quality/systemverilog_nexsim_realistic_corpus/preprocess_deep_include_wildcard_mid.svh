`include "preprocess_deep_include_wildcard_defs.svh"

module deep_wildcard_leaf(input logic [`WILDCARD_WIDTH-1:0] a, output logic [`WILDCARD_WIDTH-1:0] y);
  assign y = a;
endmodule
