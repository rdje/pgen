package defs_pkg;
  localparam logic [3:0] A = 4'b0101;
  localparam logic [3:0] B = 4'b1010;
endpackage

module imported_width_source(output logic [3:0] a, output logic [3:0] b);
  import defs_pkg::*;
  assign a = A;
  assign b = B;
endmodule

module width_chain_sink(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module multi_module_imported_width_chain_named_port(output logic [3:0] q);
  logic [3:0] s0;
  logic [3:0] s1;
  imported_width_source src(.a(s0), .b(s1));
  width_chain_sink sink(.a(s0), .b(s1), .y(q));
endmodule
