`define CHILD child
module child(input logic [3:0] a, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_module_name_internal_packed(output logic [3:0] q);
  logic [3:0] d;
  `CHILD u0(.a(d), .y(q));
endmodule
