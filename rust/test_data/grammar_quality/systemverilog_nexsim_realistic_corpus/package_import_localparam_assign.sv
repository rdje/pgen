package cfg_pkg;
  localparam int CONST = 1;
endpackage

module top(output logic y);
  import cfg_pkg::*;
  assign y = CONST;
endmodule
