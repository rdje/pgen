package cfg_pkg;
  localparam logic [3:0] A = 4'b0001;
  localparam logic [3:0] B = 4'b0010;
endpackage

module module_local_import_multi_width_assign(output logic [3:0] y0, output logic [3:0] y1);
  import cfg_pkg::*;
  assign y0 = A;
  assign y1 = B;
endmodule
