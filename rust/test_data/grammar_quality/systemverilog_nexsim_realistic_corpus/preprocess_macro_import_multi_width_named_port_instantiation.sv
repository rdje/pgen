package width_cfg_pkg;
  localparam logic [3:0] A = 4'b0011;
  localparam logic [3:0] B = 4'b1100;
endpackage

`define IMPORT_CFG import width_cfg_pkg::*;
module macro_import_width_pair(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_import_multi_width_named_port_instantiation(output logic [3:0] q);
  `IMPORT_CFG
  logic [3:0] d0;
  logic [3:0] d1;
  assign d0 = A;
  assign d1 = B;
  macro_import_width_pair u0(.a(d0), .b(d1), .y(q));
endmodule
