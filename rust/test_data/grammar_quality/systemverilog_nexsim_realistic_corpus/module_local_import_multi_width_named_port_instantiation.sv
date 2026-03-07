package cfg_pkg;
  localparam logic [3:0] A = 4'b0001;
  localparam logic [3:0] B = 4'b0010;
endpackage

module width_pair_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module module_local_import_multi_width_named_port_instantiation(output logic [3:0] q);
  import cfg_pkg::*;
  logic [3:0] d0;
  logic [3:0] d1;
  assign d0 = A;
  assign d1 = B;
  width_pair_child u0(.a(d0), .b(d1), .y(q));
endmodule
