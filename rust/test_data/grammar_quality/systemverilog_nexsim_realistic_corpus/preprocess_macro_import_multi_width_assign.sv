package width_cfg_pkg;
  localparam logic [3:0] A = 4'b0011;
  localparam logic [3:0] B = 4'b1100;
endpackage

`define IMPORT_CFG import width_cfg_pkg::*;
module preprocess_macro_import_multi_width_assign(output logic [3:0] y0, output logic [3:0] y1);
  `IMPORT_CFG
  assign y0 = A;
  assign y1 = B;
endmodule
