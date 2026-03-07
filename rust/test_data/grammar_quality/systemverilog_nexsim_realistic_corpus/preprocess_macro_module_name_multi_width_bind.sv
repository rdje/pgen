`define CHILD width_pair_child
module width_pair_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_module_name_multi_width_bind(input logic [3:0] d0, input logic [3:0] d1, output logic [3:0] q);
  `CHILD u0(.a(d0), .b(d1), .y(q));
endmodule
