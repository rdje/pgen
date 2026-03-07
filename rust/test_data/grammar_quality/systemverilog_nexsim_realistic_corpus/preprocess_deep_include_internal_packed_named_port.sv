`include "preprocess_deep_include_internal_packed_named_port_leaf.svh"

module preprocess_deep_include_internal_packed_named_port(output logic [`PACKED_WIDTH-1:0] q);
  logic [`PACKED_WIDTH-1:0] d;
  deep_packed_leaf u0(.a(d), .y(q));
endmodule
