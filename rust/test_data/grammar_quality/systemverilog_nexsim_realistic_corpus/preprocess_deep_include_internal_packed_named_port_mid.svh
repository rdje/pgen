`include "preprocess_deep_include_internal_packed_named_port_defs.svh"

module deep_packed_leaf(input logic [`PACKED_WIDTH-1:0] a, output logic [`PACKED_WIDTH-1:0] y);
  assign y = a;
endmodule
