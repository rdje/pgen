package inst_cfg_pkg;
  localparam int CONST = 1;
endpackage

module imported_cfg_leaf(output int y);
  import inst_cfg_pkg::*;
  assign y = CONST;
endmodule

module package_import_named_port_instantiation;
  int sig;
  imported_cfg_leaf u0(.y(sig));
endmodule
