package cfg_pkg;
  localparam int A = 1;
  localparam int B = 2;
endpackage

module module_local_import_multi_assign(output int y0, output int y1);
  import cfg_pkg::*;
  assign y0 = A;
  assign y1 = B;
endmodule
