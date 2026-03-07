package cfg_pkg;
  localparam int A = 1;
  localparam int B = 2;
endpackage

`define IMPORT_CFG import cfg_pkg::*;
module preprocess_macro_import_multi_assign(output int y0, output int y1);
  `IMPORT_CFG
  assign y0 = A;
  assign y1 = B;
endmodule
