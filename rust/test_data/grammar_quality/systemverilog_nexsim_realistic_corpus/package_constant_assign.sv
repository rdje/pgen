package cfg_pkg;
  localparam int CONST = 1;
endpackage

module top(output logic y);
  assign y = cfg_pkg::CONST;
endmodule
