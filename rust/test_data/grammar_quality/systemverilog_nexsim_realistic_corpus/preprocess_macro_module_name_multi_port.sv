`define CHILD child
module child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module preprocess_macro_module_name_multi_port(input logic d0, input logic d1, output logic q);
  `CHILD u0(.a(d0), .b(d1), .y(q));
endmodule
