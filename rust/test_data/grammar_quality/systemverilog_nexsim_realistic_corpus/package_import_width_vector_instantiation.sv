package packed_defs_pkg;
  localparam logic [3:0] VEC = 4'b1100;
endpackage

module imported_width_child(output logic [3:0] y);
  import packed_defs_pkg::*;
  assign y = VEC;
endmodule

module package_import_width_vector_instantiation(output logic [3:0] q);
  logic [3:0] tmp;
  imported_width_child u0(.y(tmp));
  assign q = tmp;
endmodule
