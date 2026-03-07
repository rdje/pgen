package macro_cfg_pkg;
  localparam int CONST = 1;
endpackage

`define IMPORT_CFG import macro_cfg_pkg::*;
module preprocess_macro_package_import(output int y);
  `IMPORT_CFG
  assign y = CONST;
endmodule
