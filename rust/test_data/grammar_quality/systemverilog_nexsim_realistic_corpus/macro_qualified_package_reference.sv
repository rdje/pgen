package cfg_pkg;
  localparam int V = 2;
endpackage

module macro_qualified_package_reference;
  int y;
  `define GOOD_PKG_REF cfg_pkg::V
  assign y = `GOOD_PKG_REF;
endmodule
