package width_defs_pkg;
  localparam logic [3:0] WIDTH_VALUE = 4'b1010;
endpackage

module width_child(output logic [3:0] y);
  import width_defs_pkg::*;
  assign y = WIDTH_VALUE;
endmodule

module module_local_import_width_vector_instantiation(output logic [3:0] q);
  logic [3:0] tmp;
  width_child u0(.y(tmp));
  assign q = tmp;
endmodule
