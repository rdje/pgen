`define CHILD child
module child(input logic [3:0] a, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_module_name_width_wildcard(output logic [3:0] y);
  logic [3:0] a;
  `CHILD u0(.*);
endmodule
