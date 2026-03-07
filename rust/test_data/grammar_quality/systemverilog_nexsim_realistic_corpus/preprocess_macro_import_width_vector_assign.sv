package macro_width_pkg;
  localparam logic [3:0] MACRO_VEC = 4'b0011;
endpackage

`define IMPORT_MACRO_WIDTH import macro_width_pkg::*;
module preprocess_macro_import_width_vector_assign(output logic [3:0] q);
  `IMPORT_MACRO_WIDTH
  assign q = MACRO_VEC;
endmodule
